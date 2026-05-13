"use client";

import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Chessboard } from 'react-chessboard';
import { Chess } from 'chess.js';
import { RotateCcw, Brain, User, Play, Search, Loader2, ChevronLeft, ChevronRight, Database } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';

type Side = 'white' | 'black' | 'random';
type MoveClass = 'brilliant' | 'great' | 'best' | 'good' | 'inaccuracy' | 'mistake' | 'blunder' | 'book';

const CLASS_COLORS: Record<MoveClass, string> = {
  brilliant: '#26C6DA', great: '#66BB6A', best: '#43A047',
  good: '#8BC34A', inaccuracy: '#FDD835', mistake: '#FF9800', blunder: '#E53935', book: '#AB47BC',
};
const CLASS_SYMBOLS: Record<MoveClass, string> = {
  brilliant: '!!', great: '!', best: '★', good: '✓',
  inaccuracy: '?!', mistake: '?', blunder: '??', book: '📖',
};

interface AnalysisMove {
  move: string; classification: MoveClass; cpLoss: number;
  evalBefore: number; evalAfter: number; engineBest: string;
  bestLine: string; evalBarWhite: number; isMate: boolean; mateIn: number;
}

export default function ChessPage() {
  const gameRef = useRef(new Chess());
  const [boardFen, setBoardFen] = useState('rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR');
  const [uciMoves, setUciMoves] = useState<string[]>([]);
  const [sanMoves, setSanMoves] = useState<string[]>([]);
  const [isThinking, setIsThinking] = useState(false);
  const [status, setStatus] = useState("Setup");
  const [gameOver, setGameOver] = useState(false);

  const [showSetup, setShowSetup] = useState(true);
  const [playerName, setPlayerName] = useState('Player');
  const [playerSide, setPlayerSide] = useState<Side>('white');
  const [actualSide, setActualSide] = useState<'white' | 'black'>('white');
  const [timeControl, setTimeControl] = useState(5);
  const [timeLeft, setTimeLeft] = useState({ white: 300, black: 300 });
  const [currentGameFile, setCurrentGameFile] = useState<string | null>(null);
  const [showDatabase, setShowDatabase] = useState(false);
  const [savedGames, setSavedGames] = useState<any[]>([]);
  const [loadingGames, setLoadingGames] = useState(false);

  const [moveFrom, setMoveFrom] = useState<string | null>(null);
  const [optionSquares, setOptionSquares] = useState<Record<string, React.CSSProperties>>({});

  // Analysis
  const [showAnalysis, setShowAnalysis] = useState(false);
  const [analyzing, setAnalyzing] = useState(false);
  const [analysisData, setAnalysisData] = useState<AnalysisMove[] | null>(null);
  const [whiteAcc, setWhiteAcc] = useState(0);
  const [blackAcc, setBlackAcc] = useState(0);
  const [whitePerf, setWhitePerf] = useState(0);
  const [blackPerf, setBlackPerf] = useState(0);
  const [selectedMoveIdx, setSelectedMoveIdx] = useState<number | null>(null);
  const [evalBarPct, setEvalBarPct] = useState(50);

  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
  }, [sanMoves, analysisData]);

  // Timer
  useEffect(() => {
    if (showSetup || gameOver) return;
    const timer = setInterval(() => {
      const side = gameRef.current.turn() === 'w' ? 'white' : 'black';
      setTimeLeft(prev => ({ ...prev, [side]: Math.max(0, prev[side] - 1) }));
    }, 1000);
    return () => clearInterval(timer);
  }, [boardFen, showSetup, gameOver]);

  useEffect(() => {
    if (!showSetup && !gameOver && (timeLeft.white === 0 || timeLeft.black === 0)) {
      finishGame(true);
    }
  }, [timeLeft, showSetup, gameOver]);

  // Arrow Key Navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (showSetup || showDatabase) return;
      if (e.key === 'ArrowLeft') {
        goToMove((selectedMoveIdx ?? uciMoves.length - 1) - 1);
      } else if (e.key === 'ArrowRight') {
        goToMove(selectedMoveIdx === null ? 0 : selectedMoveIdx + 1);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [selectedMoveIdx, uciMoves, showSetup, showDatabase]);

  // ---- Start Match ----
  const startMatch = () => {
    const side: 'white' | 'black' = playerSide === 'random'
      ? (Math.random() > 0.5 ? 'white' : 'black') : playerSide;

    gameRef.current = new Chess();
    setActualSide(side);
    setBoardFen(gameRef.current.fen());
    setSanMoves([]);
    setUciMoves([]);
    setGameOver(false);
    setShowAnalysis(false);
    setAnalysisData(null);
    setSelectedMoveIdx(null);
    setMoveFrom(null);
    setOptionSquares({});
    setTimeLeft({ white: timeControl * 60, black: timeControl * 60 });
    setShowSetup(false);
    setStatus(side === 'white' ? "Your turn" : "Prometheus thinking...");

    if (side === 'black') {
      setTimeout(() => callEngine([]), 500);
    }
  };

  // ---- Engine ----
  async function callEngine(currentUci: string[]) {
    setIsThinking(true);
    setStatus("Prometheus thinking...");
    try {
      const res = await fetch('/api/engine', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          action: 'move', moves: currentUci,
          wtime: timeLeft.white * 1000, btime: timeLeft.black * 1000,
        }),
      });
      const data = await res.json();
      if (data.move) {
        const from = data.move.substring(0, 2);
        const to = data.move.substring(2, 4);
        const promo = data.move.length === 5 ? data.move[4] : undefined;
        const result = gameRef.current.move({ from, to, promotion: promo });
        if (result) {
          setBoardFen(gameRef.current.fen());
            setUciMoves(prev => {
              const next = [...prev, data.move];
              setSelectedMoveIdx(next.length - 1);
              return next;
            });
            setSanMoves([...gameRef.current.history()]);
            if (gameRef.current.isGameOver()) { finishGame(); }
            else { setStatus("Your turn"); }
        }
      }
    } catch { setStatus("Engine error"); }
    finally { setIsThinking(false); }
  }

  // ---- Database ----
  const fetchGames = async () => {
    setLoadingGames(true);
    try {
      const res = await fetch('/api/engine', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ action: 'list' }),
      });
      const data = await res.json();
      setSavedGames(data.games || []);
    } catch { setStatus("Failed to load database"); }
    finally { setLoadingGames(false); }
  };

  const loadGame = async (filename: string) => {
    try {
      const res = await fetch('/api/engine', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ action: 'load', filename }),
      });
      const data = await res.json();
      if (data.pgn) {
        const tempGame = new Chess();
        tempGame.loadPgn(data.pgn);
        const history = tempGame.history();
        const uciHistory = tempGame.history({ verbose: true }).map(m => m.from + m.to + (m.promotion || ''));
        gameRef.current = tempGame;
        setBoardFen(tempGame.fen());
        setSanMoves(history);
        setUciMoves(uciHistory);
        setGameOver(true);
        setShowSetup(false);
        setShowDatabase(false);
        setStatus("Game Loaded");
        setCurrentGameFile(filename);
        const white = tempGame.header().White || 'Player';
        const black = tempGame.header().Black || 'Player';
        if (white === 'Prometheus v1') {
          setPlayerName(black);
          setActualSide('black');
        } else {
          setPlayerName(white);
          setActualSide('white');
        }
        setShowAnalysis(false);
        setAnalysisData(null);
        setSelectedMoveIdx(uciHistory.length - 1);
      }
    } catch { setStatus("Failed to load game"); }
  };

  // ---- Navigation ----
  const goToMove = (idx: number | null) => {
    if (idx === null || idx === -1) {
      gameRef.current.reset();
      setBoardFen(gameRef.current.fen());
      setSelectedMoveIdx(idx === -1 ? -1 : null);
      setEvalBarPct(50);
      return;
    }
    
    if (idx < 0 || idx >= uciMoves.length) return;

    const tempGame = new Chess();
    for (let i = 0; i <= idx; i++) {
      const m = uciMoves[i];
      const from = m.substring(0, 2);
      const to = m.substring(2, 4);
      const promo = m.length === 5 ? m[4] : undefined;
      tempGame.move({ from, to, promotion: promo });
    }
    setBoardFen(tempGame.fen());
    setSelectedMoveIdx(idx);
    if (analysisData && analysisData[idx]) {
      setEvalBarPct(analysisData[idx].evalBarWhite);
    }
  };

  // ---- Player Move (shared by click & drag) ----
  function tryMove(from: string, to: string): boolean {
    const turn = gameRef.current.turn();
    const isPlayerTurn = (turn === 'w' && actualSide === 'white') || (turn === 'b' && actualSide === 'black');
    if (!isPlayerTurn || gameOver || isThinking || showSetup) return false;

    try {
      const result = gameRef.current.move({ from, to, promotion: 'q' });
      if (!result) return false;

      setBoardFen(gameRef.current.fen());
      const uci = from + to + (result.promotion || '');
      const newUci = [...uciMoves, uci];
      setUciMoves(newUci);
      setSelectedMoveIdx(newUci.length - 1);
      setSanMoves([...gameRef.current.history()]);
      setMoveFrom(null);
      setOptionSquares({});

      if (gameRef.current.isGameOver()) { finishGame(); }
      else { setTimeout(() => callEngine(newUci), 200); }
      return true;
    } catch { return false; }
  }

  // ---- v5 Chessboard callbacks ----
  const handlePieceDrop = useCallback(
    ({ sourceSquare, targetSquare }: any) => tryMove(sourceSquare, targetSquare),
    [boardFen, uciMoves, gameOver, isThinking, showSetup, actualSide]
  );

  const handleSquareClick = useCallback(({ square }: any) => {
    if (gameOver || isThinking || showSetup) return;
    if (moveFrom === null) {
      const moves = gameRef.current.moves({ square, verbose: true });
      if (moves.length === 0) return;
      setMoveFrom(square);
      const h: Record<string, React.CSSProperties> = {};
      h[square] = { background: 'rgba(255,214,0,0.6)' };
      moves.forEach(m => {
        h[m.to] = { background: gameRef.current.get(m.to as any)
          ? 'radial-gradient(circle, rgba(255,0,0,0.5) 85%, transparent 85%)'
          : 'radial-gradient(circle, rgba(0,230,118,0.6) 25%, transparent 25%)' };
      });
      setOptionSquares(h);
    } else {
      if (!tryMove(moveFrom, square)) {
        setMoveFrom(null); setOptionSquares({});
        const moves = gameRef.current.moves({ square, verbose: true });
        if (moves.length > 0) {
          setMoveFrom(square);
          const h: Record<string, React.CSSProperties> = {};
          h[square] = { background: 'rgba(255,214,0,0.6)' };
          moves.forEach(m => {
            h[m.to] = { background: gameRef.current.get(m.to as any)
              ? 'radial-gradient(circle, rgba(255,0,0,0.5) 85%, transparent 85%)'
              : 'radial-gradient(circle, rgba(0,230,118,0.6) 25%, transparent 25%)' };
          });
          setOptionSquares(h);
        }
      }
    }
  }, [boardFen, moveFrom, gameOver, isThinking, showSetup, actualSide, uciMoves]);

  // ---- Game Over & PGN ----
  async function finishGame(timeout = false) {
    setGameOver(true);
    const g = gameRef.current;
    let result = '*';
    let reason = 'Game Over';
    if (timeout) {
      reason = 'Time Out!';
      result = g.turn() === 'w' ? '0-1' : '1-0';
    } else if (g.isCheckmate()) {
      reason = 'Checkmate!';
      result = g.turn() === 'w' ? '0-1' : '1-0';
    } else if (g.isDraw()) {
      reason = 'Draw'; result = '1/2-1/2';
    }
    setStatus(reason);

    // Build PGN with proper headers
    const now = new Date();
    const white = actualSide === 'white' ? playerName : 'Prometheus v1';
    const black = actualSide === 'black' ? playerName : 'Prometheus v1';
    const dateStr = `${now.getFullYear()}.${String(now.getMonth()+1).padStart(2,'0')}.${String(now.getDate()).padStart(2,'0')}`;
    const timeStr = `${String(now.getHours()).padStart(2,'0')}:${String(now.getMinutes()).padStart(2,'0')}:${String(now.getSeconds()).padStart(2,'0')}`;

    g.header('Event', 'Prometheus Arena');
    g.header('Site', 'localhost');
    g.header('Date', dateStr);
    g.header('Round', '1');
    g.header('White', white);
    g.header('Black', black);
    g.header('Result', result);
    g.header('TimeControl', `${timeControl * 60}`);
    g.header('Time', timeStr);

    const pgn = g.pgn();
    const res = await fetch('/api/engine', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ action: 'save', pgn, playerName, playerSide: actualSide, timeControl, result }),
    });
    const data = await res.json();
    if (data.saved) setCurrentGameFile(data.saved);
  }

  // ---- Analysis ----
  async function runAnalysis() {
    setAnalyzing(true);
    setShowAnalysis(true);
    try {
      const res = await fetch('/api/analyze', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ uciMoves }),
      });
      const data = await res.json();
      setAnalysisData(data.analysis);
      setWhiteAcc(data.whiteAccuracy);
      setBlackAcc(data.blackAccuracy);
      
      // Calculate performance rating (approximate)
      const calcPerf = (acc: number) => Math.round(acc * acc * 0.25 + 500); // 100% -> 3000, 50% -> 1125, 80% -> 2100
      setWhitePerf(calcPerf(data.whiteAccuracy));
      setBlackPerf(calcPerf(data.blackAccuracy));

      if (data.analysis && data.analysis.length > 0) {
        const lastIdx = data.analysis.length - 1;
        setSelectedMoveIdx(lastIdx);
        setEvalBarPct(data.analysis[lastIdx].evalBarWhite);
      }

      // Automatically save analysis for Prometheus to learn from
      if (currentGameFile) {
        await fetch('/api/engine', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ 
            action: 'save-analysis', 
            filename: currentGameFile, 
            analysisData: {
              moves: uciMoves,
              analysis: data.analysis,
              whiteAccuracy: data.whiteAccuracy,
              blackAccuracy: data.blackAccuracy,
              whitePerformance: calcPerf(data.whiteAccuracy),
              blackPerformance: calcPerf(data.blackAccuracy),
              timestamp: new Date().toISOString()
            }
          }),
        }).catch(err => console.error("Failed to save analysis:", err));
      }
    } catch { setStatus("Analysis failed"); }
    finally { setAnalyzing(false); }
  }

  const fmt = (s: number) => `${Math.floor(s / 60)}:${(s % 60).toString().padStart(2, '0')}`;

  // ---- RENDER ----
  return (
    <div className="min-h-screen bg-[#F0F0F0] text-black font-mono p-4 md:p-8 flex items-center justify-center">
      {/* SETUP MODAL */}
      <AnimatePresence>
        {showSetup && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
            className="fixed inset-0 bg-black/60 z-50 flex items-center justify-center p-4">
            <motion.div initial={{ y: 50 }} animate={{ y: 0 }}
              className="bg-white border-[6px] border-black shadow-[16px_16px_0px_#000] p-10 max-w-lg w-full">
              <h1 className="text-4xl font-black mb-8 border-b-[6px] border-black pb-4 uppercase">Match Setup</h1>
              <div className="space-y-6">
                {/* Player Name */}
                <div>
                  <label className="text-lg font-black mb-2 block uppercase tracking-widest">Your Name</label>
                  <input type="text" value={playerName} onChange={e => setPlayerName(e.target.value)}
                    className="w-full py-3 px-4 border-[4px] border-black font-bold text-xl bg-white outline-none focus:bg-[#FFD600]/20"
                    placeholder="Enter your name..." />
                </div>
                {/* Side */}
                <div>
                  <label className="text-lg font-black mb-2 block uppercase tracking-widest">Pick Side</label>
                  <div className="grid grid-cols-3 gap-3">
                    {(['white', 'random', 'black'] as Side[]).map(s => (
                      <button key={s} onClick={() => setPlayerSide(s)}
                        className={`py-3 border-[4px] border-black font-black uppercase transition-all ${
                          playerSide === s ? 'bg-[#FF80BF] shadow-none translate-x-1 translate-y-1'
                            : 'bg-white shadow-[6px_6px_0px_#000] hover:-translate-x-1 hover:-translate-y-1'}`}>
                        {s}
                      </button>
                    ))}
                  </div>
                </div>
                {/* Time */}
                <div>
                  <label className="text-lg font-black mb-2 block uppercase tracking-widest">Time (Min)</label>
                  <div className="grid grid-cols-4 gap-3">
                    {[1, 3, 5, 10].map(t => (
                      <button key={t} onClick={() => setTimeControl(t)}
                        className={`py-3 border-[4px] border-black font-black transition-all ${
                          timeControl === t ? 'bg-[#00E676] shadow-none translate-x-1 translate-y-1'
                            : 'bg-white shadow-[6px_6px_0px_#000] hover:-translate-x-1 hover:-translate-y-1'}`}>
                        {t}m
                      </button>
                    ))}
                  </div>
                </div>
                <button onClick={startMatch}
                  className="w-full mt-4 py-5 bg-[#FFD600] border-[6px] border-black text-2xl font-black uppercase shadow-[10px_10px_0px_#000] hover:-translate-x-1 hover:-translate-y-1 active:translate-x-1 active:translate-y-1 active:shadow-none transition-all flex items-center justify-center gap-4">
                  <Play size={36} strokeWidth={3} /> START
                </button>
                
                <div className="relative">
                  <div className="absolute inset-0 flex items-center"><span className="w-full border-t-[4px] border-black"></span></div>
                  <div className="relative flex justify-center"><span className="bg-white px-4 font-black uppercase text-xs">or review</span></div>
                </div>

                <button onClick={() => { setShowDatabase(true); fetchGames(); }}
                  className="w-full py-4 bg-[#26C6DA] border-[6px] border-black text-xl font-black uppercase shadow-[10px_10px_0px_#000] hover:-translate-x-1 hover:-translate-y-1 active:translate-x-1 active:translate-y-1 active:shadow-none transition-all flex items-center justify-center gap-4 text-white">
                  <Database size={28} strokeWidth={3} /> OPEN DATABASE
                </button>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* DATABASE MODAL */}
      <AnimatePresence>
        {showDatabase && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
            className="fixed inset-0 bg-black/60 z-50 flex items-center justify-center p-4">
            <motion.div initial={{ y: 50 }} animate={{ y: 0 }}
              className="bg-white border-[6px] border-black shadow-[16px_16px_0px_#000] p-8 max-w-2xl w-full max-h-[80vh] flex flex-col">
              <div className="flex justify-between items-center mb-6 border-b-[6px] border-black pb-4">
                <h1 className="text-3xl font-black uppercase flex items-center gap-3">
                  <Database size={32} /> Game Database
                </h1>
                <button onClick={() => setShowDatabase(false)} className="text-3xl font-black hover:scale-110 transition-transform">✕</button>
              </div>
              
              <div className="flex-1 overflow-y-auto space-y-4 pr-2 custom-scrollbar">
                {loadingGames ? (
                   <div className="flex flex-col items-center justify-center p-10 gap-4">
                     <Loader2 className="animate-spin" size={48} />
                     <span className="font-black uppercase text-sm">Loading Games...</span>
                   </div>
                ) : savedGames.length === 0 ? (
                   <div className="text-center py-20 border-[4px] border-dashed border-black/20">
                     <p className="text-xl font-black opacity-30 uppercase tracking-widest">No saved games found</p>
                   </div>
                ) : (
                  savedGames.map((g, i) => (
                    <div key={i} onClick={() => loadGame(g.filename)}
                      className="border-[4px] border-black p-5 cursor-pointer hover:bg-[#FFD600] shadow-[6px_6px_0px_#000] hover:shadow-none hover:translate-x-1 hover:translate-y-1 transition-all flex justify-between items-center group">
                      <div className="flex flex-col gap-1">
                        <div className="font-black text-xl uppercase tracking-tight">{g.white} <span className="opacity-30">vs</span> {g.black}</div>
                        <div className="flex items-center gap-3">
                          <span className="text-[10px] bg-black text-white px-2 py-0.5 font-black uppercase">{g.result}</span>
                          <span className="text-[10px] font-black opacity-40 uppercase tracking-widest">{g.date}</span>
                        </div>
                      </div>
                      <div className="bg-black text-white p-2 group-hover:bg-white group-hover:text-black transition-colors">
                        <Play size={20} fill="currentColor" />
                      </div>
                    </div>
                  ))
                )}
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* MAIN LAYOUT */}
      <div className="max-w-7xl w-full grid grid-cols-1 lg:grid-cols-12 gap-10 items-start">
        <div className="lg:col-span-7 flex flex-col gap-5">
          {/* Opponent clock */}
          <div className={`p-4 border-[6px] border-black flex justify-between items-center shadow-[8px_8px_0px_#000] transition-colors ${
            gameRef.current.turn() === (actualSide === 'white' ? 'b' : 'w') ? 'bg-[#FF80BF]' : 'bg-white'}`}>
            <div className="flex items-center gap-3">
              <span className="font-black text-lg uppercase">Prometheus</span>
            </div>
            <span className="text-3xl font-black tabular-nums">{fmt(actualSide === 'white' ? timeLeft.black : timeLeft.white)}</span>
          </div>

          {/* Board + Eval Bar */}
          <div className="flex gap-0">
            {/* Eval Bar */}
            {showAnalysis && (
              <div className="w-8 border-[4px] border-black border-r-0 flex flex-col overflow-hidden" style={{ minHeight: '100%' }}>
                <div className="bg-[#1a1a1a] transition-all duration-500" style={{ height: `${actualSide === 'white' ? 100 - evalBarPct : evalBarPct}%` }}>
                  {evalBarPct < 50 && (
                    <div className="text-white text-[9px] font-black text-center pt-1">
                      {selectedMoveIdx !== null && analysisData?.[selectedMoveIdx]?.isMate ? 'M' : Math.abs(Math.round((50 - evalBarPct) / 5 * 10) / 10)}
                    </div>
                  )}
                </div>
                <div className="bg-[#EEEEEE] flex-1">
                  {evalBarPct >= 50 && (
                    <div className="text-black text-[9px] font-black text-center pt-1">
                      {selectedMoveIdx !== null && analysisData?.[selectedMoveIdx]?.isMate ? 'M' : Math.abs(Math.round((evalBarPct - 50) / 5 * 10) / 10)}
                    </div>
                  )}
                </div>
              </div>
            )}
            {/* Board */}
            <div className="flex-1 border-[4px] border-black bg-white shadow-[16px_16px_0px_#000]">
              <Chessboard options={{
                id: 'prometheus',
                position: boardFen,
                boardOrientation: actualSide,
                onPieceDrop: handlePieceDrop,
                onSquareClick: handleSquareClick,
                darkSquareStyle: { backgroundColor: '#B58863' },
                lightSquareStyle: { backgroundColor: '#F0D9B5' },
                squareStyles: optionSquares,
                animationDurationInMs: 150,
                allowDragging: true,
              }} />
            </div>
          </div>

          {/* Navigation Buttons */}
          <div className="grid grid-cols-2 gap-4">
            <button 
              onClick={() => goToMove((selectedMoveIdx ?? uciMoves.length - 1) - 1)}
              disabled={selectedMoveIdx === -1 || uciMoves.length === 0}
              className="py-3 bg-white border-[4px] border-black shadow-[6px_6px_0px_#000] hover:-translate-x-1 hover:-translate-y-1 active:translate-x-1 active:translate-y-1 active:shadow-none transition-all flex items-center justify-center gap-2 font-black uppercase disabled:opacity-50"
            >
              <ChevronLeft size={24} strokeWidth={3} /> Previous
            </button>
            <button 
              onClick={() => goToMove(selectedMoveIdx === null ? 0 : selectedMoveIdx + 1)}
              disabled={selectedMoveIdx === uciMoves.length - 1 || uciMoves.length === 0}
              className="py-3 bg-white border-[4px] border-black shadow-[6px_6px_0px_#000] hover:-translate-x-1 hover:-translate-y-1 active:translate-x-1 active:translate-y-1 active:shadow-none transition-all flex items-center justify-center gap-2 font-black uppercase disabled:opacity-50"
            >
              Next <ChevronRight size={24} strokeWidth={3} />
            </button>
          </div>

          {/* Engine Line (shown during analysis) */}
          {showAnalysis && analysisData && selectedMoveIdx !== null && selectedMoveIdx !== -1 && (
            <div className="bg-[#1a1a1a] border-[4px] border-black p-4 shadow-[8px_8px_0px_#000] text-white">
              <div className="flex justify-between items-center mb-2">
                <span className="text-[10px] font-black uppercase tracking-widest text-[#FFD600]">Stockfish Analysis</span>
                <span className="text-xl font-black">
                  {analysisData[selectedMoveIdx]?.isMate
                    ? `#${analysisData[selectedMoveIdx].mateIn}`
                    : `${analysisData[selectedMoveIdx].evalAfter > 0 ? '+' : ''}${(analysisData[selectedMoveIdx].evalAfter / 100).toFixed(1)}`}
                </span>
              </div>
              <div className="font-mono text-xs opacity-60 leading-relaxed break-all">
                {analysisData[selectedMoveIdx]?.bestLine || 'N/A'}
              </div>
            </div>
          )}

          {/* Player clock */}
          <div className={`p-4 border-[6px] border-black flex justify-between items-center shadow-[8px_8px_0px_#000] transition-colors ${
            gameRef.current.turn() === (actualSide === 'white' ? 'w' : 'b') ? 'bg-[#00E676]' : 'bg-white'}`}>
            <div className="flex items-center gap-3">
              <span className="font-black text-lg uppercase">{playerName}</span>
            </div>
            <span className="text-3xl font-black tabular-nums">{fmt(actualSide === 'white' ? timeLeft.white : timeLeft.black)}</span>
          </div>
        </div>

        {/* RIGHT SIDEBAR */}
        <div className="lg:col-span-5 flex flex-col gap-6 self-stretch">
          {/* Status */}
          <div className="bg-[#FFD600] border-[6px] border-black p-6 shadow-[10px_10px_0px_#000]">
            <h2 className="text-[10px] font-black uppercase tracking-[0.3em] mb-2 border-b-[3px] border-black pb-1">Status</h2>
            <p className="text-2xl font-black italic uppercase">{status}</p>
            {isThinking && <div className="mt-2 h-2 bg-black/20 overflow-hidden"><div className="h-full w-1/3 bg-black animate-pulse" /></div>}
          </div>

          {/* Analysis Accuracy */}
          {showAnalysis && analysisData && (
            <div className="bg-white border-[6px] border-black p-6 shadow-[10px_10px_0px_#000]">
              <h2 className="text-[10px] font-black uppercase tracking-[0.3em] mb-4 border-b-[3px] border-black pb-1">Performance</h2>
              <div className="grid grid-cols-2 gap-4">
                <div className="border-[3px] border-black p-3">
                  <div className="text-[10px] font-black uppercase opacity-40">White</div>
                  <div className="text-3xl font-black">{whiteAcc}%</div>
                  <div className="text-sm font-black text-[#43A047]">Est. {whitePerf}</div>
                </div>
                <div className="border-[3px] border-black p-3 bg-black text-white">
                  <div className="text-[10px] font-black uppercase opacity-40">Black</div>
                  <div className="text-3xl font-black">{blackAcc}%</div>
                  <div className="text-sm font-black text-[#26C6DA]">Est. {blackPerf}</div>
                </div>
              </div>
            </div>
          )}

          {/* Move History */}
          <div className="flex-1 bg-white border-[6px] border-black p-6 shadow-[10px_10px_0px_#000] flex flex-col min-h-[300px] max-h-[500px]">
            <h2 className="text-[10px] font-black uppercase tracking-[0.3em] mb-4 border-b-[3px] border-black pb-1">Moves</h2>
            <div ref={scrollRef} className="flex-1 overflow-y-auto text-sm font-bold">
              {sanMoves.length === 0
                ? <div className="h-full flex items-center justify-center text-black/10 text-3xl uppercase font-black">Waiting</div>
                : (
                  <table className="w-full">
                    <tbody>
                      {Array.from({ length: Math.ceil(sanMoves.length / 2) }).map((_, i) => {
                        const wIdx = i * 2;
                        const bIdx = i * 2 + 1;
                        const wA = analysisData?.[wIdx];
                        const bA = analysisData?.[bIdx];
                        return (
                          <tr key={i} className="border-b border-black/10">
                            <td className="py-1.5 pr-2 w-8 text-[10px] opacity-30 font-black">{i + 1}.</td>
                            <td className={`py-1.5 pr-1 cursor-pointer hover:bg-black/5 ${selectedMoveIdx === wIdx ? 'bg-[#FFD600]/30' : ''}`}
                              onClick={() => goToMove(wIdx)}>
                              <span>{sanMoves[wIdx]}</span>
                              {wA && (
                                <span className="ml-1 text-[10px] font-black px-1 py-0.5 inline-block rounded-sm"
                                  style={{ backgroundColor: CLASS_COLORS[wA.classification as MoveClass], color: '#fff' }}>
                                  {CLASS_SYMBOLS[wA.classification as MoveClass]}
                                </span>
                              )}
                            </td>
                            <td className={`py-1.5 cursor-pointer hover:bg-black/5 ${selectedMoveIdx === bIdx ? 'bg-[#FFD600]/30' : ''}`}
                              onClick={() => { if (sanMoves[bIdx]) goToMove(bIdx); }}>
                              {sanMoves[bIdx] && (
                                <>
                                  <span>{sanMoves[bIdx]}</span>
                                  {bA && (
                                    <span className="ml-1 text-[10px] font-black px-1 py-0.5 inline-block rounded-sm"
                                      style={{ backgroundColor: CLASS_COLORS[bA.classification as MoveClass], color: '#fff' }}>
                                      {CLASS_SYMBOLS[bA.classification as MoveClass]}
                                    </span>
                                  )}
                                </>
                              )}
                            </td>
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                )}
            </div>
          </div>

          {/* Buttons */}
          <div className="flex flex-col gap-3">
            {gameOver && !showAnalysis && (
              <button onClick={runAnalysis} disabled={analyzing}
                className="py-4 bg-[#26C6DA] text-white border-[6px] border-black text-xl font-black uppercase shadow-[10px_10px_0px_#000] hover:-translate-x-1 hover:-translate-y-1 active:translate-x-1 active:translate-y-1 active:shadow-none transition-all flex items-center justify-center gap-3 disabled:opacity-50">
                {analyzing ? <><Loader2 size={24} className="animate-spin" /> Analyzing...</> : <><Search size={24} /> Analyze Game</>}
              </button>
            )}
            <button onClick={() => { setShowSetup(true); setShowAnalysis(false); setAnalysisData(null); }}
              className="py-4 bg-black text-white border-[6px] border-black text-xl font-black uppercase shadow-[10px_10px_0px_#FFD600] hover:translate-x-1 hover:translate-y-1 hover:shadow-none transition-all flex items-center justify-center gap-3">
              <RotateCcw size={24} /> New Match
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
