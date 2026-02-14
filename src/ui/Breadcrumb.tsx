import React from "react";
import { Box, Text } from "ink";

interface Props {
  breadcrumbs: string[];
  canGoBack: boolean;
}

export function Breadcrumb({ breadcrumbs, canGoBack }: Props) {
  if (breadcrumbs.length <= 1) return null;

  return (
    <Box paddingX={1}>
      {breadcrumbs.map((crumb, i) => {
        const isLast = i === breadcrumbs.length - 1;
        return (
          <React.Fragment key={i}>
            {i > 0 && <Text dimColor> &gt; </Text>}
            <Text dimColor={!isLast} bold={isLast}>{crumb}</Text>
          </React.Fragment>
        );
      })}
      {canGoBack && <Text dimColor>  [esc/backspace] back</Text>}
    </Box>
  );
}
