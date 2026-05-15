import { NextRequest, NextResponse } from "next/server";
import { spawn } from "child_process";
import path from "path";
import fs from "fs";

const ENGINE_PATH = path.join(
  process.cwd(),
  "..",
  "target",
  "release",
  "prometheus",
);
const PLAYED_GAMES_DIR = "/home/ansh/prometheus/games/played";
const ANALYZED_GAMES_DIR = "/home/ansh/prometheus/games/analyzedplayed";

export async function POST(req: NextRequest) {
  const body = await req.json();

  // Save Analysis
  if (body.action === "save-analysis") {
    const { filename, analysisData } = body;
    if (!fs.existsSync(ANALYZED_GAMES_DIR)) {
      fs.mkdirSync(ANALYZED_GAMES_DIR, { recursive: true });
    }
    const analyzedFilename = filename.replace(".pgn", ".json");
    fs.writeFileSync(
      path.join(ANALYZED_GAMES_DIR, analyzedFilename),
      JSON.stringify(analysisData, null, 2),
    );
    return NextResponse.json({ success: true, saved: analyzedFilename });
  }

  // Save PGN
  if (body.action === "save") {
    const { pgn, playerName, playerSide, timeControl, result } = body;
    if (!fs.existsSync(PLAYED_GAMES_DIR)) {
      fs.mkdirSync(PLAYED_GAMES_DIR, { recursive: true });
    }
    const now = new Date();
    const dateStr = now.toISOString().replace(/[:.]/g, "-");
    const filename = `game-${dateStr}.pgn`;
    fs.writeFileSync(path.join(PLAYED_GAMES_DIR, filename), pgn);
    return NextResponse.json({ success: true, saved: filename });
  }

  // List games
  if (body.action === "list") {
    if (!fs.existsSync(PLAYED_GAMES_DIR))
      return NextResponse.json({ games: [] });
    const files = fs
      .readdirSync(PLAYED_GAMES_DIR)
      .filter((f) => f.endsWith(".pgn"));
    const games = files.map((f) => {
      const content = fs.readFileSync(path.join(PLAYED_GAMES_DIR, f), "utf-8");
      const white = content.match(/\[White "(.+)"\]/)?.[1] || "Unknown";
      const black = content.match(/\[Black "(.+)"\]/)?.[1] || "Unknown";
      const result = content.match(/\[Result "(.+)"\]/)?.[1] || "*";
      const date = content.match(/\[Date "(.+)"\]/)?.[1] || "";
      return { filename: f, white, black, result, date };
    });
    return NextResponse.json({
      games: games.sort((a, b) => b.filename.localeCompare(a.filename)),
    });
  }

  // Load game
  if (body.action === "load") {
    const { filename } = body;
    const filePath = path.join(PLAYED_GAMES_DIR, filename);
    if (!fs.existsSync(filePath))
      return NextResponse.json({ error: "File not found" }, { status: 404 });
    const pgn = fs.readFileSync(filePath, "utf-8");
    return NextResponse.json({ pgn });
  }

  // Get engine move
  if (body.action === "move") {
    const { moves, wtime, btime } = body;
    return new Promise((resolve) => {
      const engine = spawn(ENGINE_PATH);
      let bestMove = "";
      let openingName = "";
      let output = "";

      engine.stdout.on("data", (data) => {
        output += data.toString();
        const lines = output.split("\n");
        for (const line of lines) {
          if (line.startsWith("bestmove")) {
            bestMove = line.split(" ")[1]?.trim();
            engine.stdin.write("quit\n");
          }
          if (line.startsWith("info string opening ")) {
            openingName = line.replace("info string opening ", "").trim();
          }
        }
      });

      engine.on("error", () => {
        resolve(NextResponse.json({ error: "Engine failed" }, { status: 500 }));
      });

      engine.on("close", () => {
        resolve(NextResponse.json({ move: bestMove, opening: openingName }));
      });

      engine.stdin.write("uci\n");
      engine.stdin.write("isready\n");
      engine.stdin.write("ucinewgame\n");

      if (moves && moves.length > 0) {
        engine.stdin.write(`position startpos moves ${moves.join(" ")}\n`);
      } else {
        engine.stdin.write("position startpos\n");
      }

      if (wtime && btime) {
        engine.stdin.write(`go wtime ${wtime} btime ${btime}\n`);
      } else {
        engine.stdin.write("go movetime 500\n");
      }
    });
  }

  return NextResponse.json({ error: "Unknown action" }, { status: 400 });
}
