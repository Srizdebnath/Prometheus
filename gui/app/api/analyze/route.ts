import { NextRequest, NextResponse } from 'next/server';
import { spawn } from 'child_process';
import path from 'path';

const ENGINE_PATH = '/usr/games/stockfish';
const DEPTH = 15;

interface RawEval {
  score: number;    // centipawns from side-to-move perspective
  isMate: boolean;
  mateIn: number;   // mate in N moves (positive = winning, negative = losing)
  bestMove: string;
  bestLine: string;  // full PV line
}

function parseEngineOutput(infoLines: string[]): RawEval {
  let score = 0, isMate = false, mateIn = 0, bestLine = '';

  // Use the deepest info line
  for (let i = infoLines.length - 1; i >= 0; i--) {
    const line = infoLines[i];
    const mateMatch = line.match(/score mate (-?\d+)/);
    const cpMatch = line.match(/score cp (-?\d+)/);
    const pvMatch = line.match(/ pv (.+)/);

    if (mateMatch && !isMate) {
      isMate = true;
      mateIn = parseInt(mateMatch[1]);
      score = mateIn > 0 ? 100000 - mateIn : -100000 - mateIn;
    }
    if (cpMatch && !isMate) {
      score = parseInt(cpMatch[1]);
    }
    if (pvMatch && !bestLine) {
      bestLine = pvMatch[1].trim();
    }
  }

  return { score, isMate, mateIn, bestMove: '', bestLine };
}

export async function POST(req: NextRequest) {
  const { uciMoves } = await req.json();

  return new Promise((resolve) => {
    const engine = spawn(ENGINE_PATH);
    let buffer = '';
    const evaluations: RawEval[] = [];
    let currentInfoLines: string[] = [];
    let positionIndex = 0;
    const totalPositions = uciMoves.length + 1;
    let ready = false;

    engine.stdout.on('data', (data) => {
      buffer += data.toString();
      const lines = buffer.split('\n');
      buffer = lines.pop() || '';

      for (const line of lines) {
        if (line === 'readyok' && !ready) {
          ready = true;
          engine.stdin.write('position startpos\n');
          engine.stdin.write(`go depth ${DEPTH}\n`);
          continue;
        }

        if (line.startsWith('info') && line.includes('score')) {
          currentInfoLines.push(line);
        }

        if (line.startsWith('bestmove')) {
          const bestMove = line.split(' ')[1]?.trim() || '';
          const parsed = parseEngineOutput(currentInfoLines);
          parsed.bestMove = bestMove;
          evaluations.push(parsed);
          currentInfoLines = [];
          positionIndex++;

          if (positionIndex >= totalPositions) {
            engine.stdin.write('quit\n');
          } else {
            const movesForPos = uciMoves.slice(0, positionIndex);
            if (movesForPos.length > 0) {
              engine.stdin.write(`position startpos moves ${movesForPos.join(' ')}\n`);
            } else {
              engine.stdin.write('position startpos\n');
            }
            engine.stdin.write(`go depth ${DEPTH}\n`);
          }
        }
      }
    });

    engine.on('close', () => {
      const analysis = [];

      for (let i = 0; i < uciMoves.length; i++) {
        const isWhiteMove = i % 2 === 0;
        const evalBefore = evaluations[i];
        const evalAfter = evaluations[i + 1];

        if (!evalBefore || !evalAfter) {
          analysis.push({
            move: uciMoves[i],
            evalBefore: 0, evalAfter: 0,
            evalBarWhite: 50,
            cpLoss: 0, classification: 'good',
            engineBest: '', bestLine: '',
            isMate: false, mateIn: 0,
          });
          continue;
        }

        // Normalize evals to WHITE's perspective (always)
        const whiteEvalBefore = isWhiteMove ? evalBefore.score : -evalBefore.score;
        const whiteEvalAfter = isWhiteMove ? -evalAfter.score : evalAfter.score;

        // cpLoss from the mover's perspective
        const actualLoss = Math.max(0, isWhiteMove ? (whiteEvalBefore - whiteEvalAfter) : (whiteEvalAfter - whiteEvalBefore));

        // Classify the move
        let classification: string;

        // If this move delivers checkmate → always best
        if (evalAfter.isMate && evalAfter.mateIn === 0) {
          classification = 'best';
        }
        // If before we had a forced mate and we kept it (or found shorter mate) → best
        else if (evalBefore.isMate && evalBefore.mateIn > 0 && evalAfter.isMate && evalAfter.mateIn <= 0) {
          // We had mate and opponent now also sees mate against them
          classification = 'best';
        }
        // If we had mate in N and now it's gone → blunder
        else if (evalBefore.isMate && evalBefore.mateIn > 0 && !evalAfter.isMate) {
          classification = 'blunder';
        }
        // If we walked into getting mated
        else if (!evalBefore.isMate && evalAfter.isMate && evalAfter.mateIn > 0) {
          classification = 'blunder';
        }
        // Normal centipawn loss classification
        else if (actualLoss <= 0) {
          classification = 'best';
        } else if (actualLoss <= 10) {
          classification = 'great';
        } else if (actualLoss <= 25) {
          classification = 'good';
        } else if (actualLoss <= 50) {
          classification = 'inaccuracy';
        } else if (actualLoss <= 150) {
          classification = 'mistake';
        } else {
          classification = 'blunder';
        }

        // Check for brilliant: we sacrificed material (eval dipped) but the move is still best/great
        if ((classification === 'best' || classification === 'great') && 
            evalBefore.bestMove !== uciMoves[i] && 
            Math.abs(whiteEvalAfter) > Math.abs(whiteEvalBefore) + 100) {
          classification = 'brilliant';
        }

        // Eval bar: convert to win% for white (0-100)
        const evalForBar = whiteEvalAfter;
        const winPct = evalAfter.isMate
          ? (evalAfter.mateIn <= 0 ? (isWhiteMove ? 100 : 0) : (isWhiteMove ? 0 : 100))
          : Math.min(100, Math.max(0, 50 + 50 * (2 / (1 + Math.exp(-0.004 * evalForBar)) - 1)));

        analysis.push({
          move: uciMoves[i],
          evalBefore: whiteEvalBefore,
          evalAfter: whiteEvalAfter,
          evalBarWhite: Math.round(winPct),
          cpLoss: actualLoss,
          classification,
          engineBest: evalBefore.bestMove,
          bestLine: evalBefore.bestLine,
          isMate: evalAfter.isMate,
          mateIn: evalAfter.mateIn,
        });
      }

      // Accuracy using Lichess-style sigmoid
      const whiteAccs: number[] = [];
      const blackAccs: number[] = [];
      analysis.forEach((a, i) => {
        const acc = Math.max(0, 103.1668 * Math.exp(-0.04354 * a.cpLoss) - 3.1668);
        if (i % 2 === 0) whiteAccs.push(acc);
        else blackAccs.push(acc);
      });

      const avg = (arr: number[]) => arr.length > 0 ? arr.reduce((a, b) => a + b, 0) / arr.length : 0;

      resolve(NextResponse.json({
        analysis,
        whiteAccuracy: Math.round(avg(whiteAccs) * 10) / 10,
        blackAccuracy: Math.round(avg(blackAccs) * 10) / 10,
      }));
    });

    engine.on('error', () => {
      resolve(NextResponse.json({ error: 'Engine failed' }, { status: 500 }));
    });

    engine.stdin.write('uci\n');
    engine.stdin.write('isready\n');
  });
}
