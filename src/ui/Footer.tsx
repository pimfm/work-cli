import React from "react";
import { Box, Text } from "ink";

interface Props {
  canGoBack?: boolean;
}

export function Footer({ canGoBack }: Props) {
  return (
    <Box paddingX={1}>
      <Text dimColor>
        {canGoBack ? "[esc/b] back  " : ""}
        [↑/↓] navigate  [enter] start/stop  [d] dispatch  [a] agents  [t] time  [r] refresh  [c] complete  {canGoBack ? "" : "[q/esc] quit"}
      </Text>
    </Box>
  );
}
