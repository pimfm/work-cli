import React from "react";
import { render } from "ink";
import { Start } from "./commands/start.js";
import { TimeStatsCommand } from "./commands/time-stats.js";

const command = process.argv[2];
const subcommand = process.argv[3];

if (command === "time" && subcommand === "stats") {
  const { waitUntilExit } = render(<TimeStatsCommand />);
  waitUntilExit();
} else if (command === "start" || !command) {
  // Enter alternate screen buffer and hide cursor
  process.stdout.write("\x1b[?1049h");
  process.stdout.write("\x1b[?25l");

  const { waitUntilExit } = render(<Start />);

  waitUntilExit().then(() => {
    // Show cursor and exit alternate screen buffer
    process.stdout.write("\x1b[?25h");
    process.stdout.write("\x1b[?1049l");
  });
} else {
  console.error(`Unknown command: ${command}`);
  console.error("Usage: work [start] | work time stats");
  process.exit(1);
}
