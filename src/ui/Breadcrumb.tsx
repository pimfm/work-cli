import React from "react";
import { Text } from "ink";

interface Props {
  items: string[];
  forwardItems?: string[];
}

export function Breadcrumb({ items, forwardItems = [] }: Props) {
  const hasForward = forwardItems.length > 0;
  if (items.length <= 1 && !hasForward) return null;

  return (
    <Text>
      {items.map((item, i) => {
        const isLast = i === items.length - 1;
        return (
          <Text key={i}>
            {isLast ? (
              <Text bold color="cyan">{item}</Text>
            ) : (
              <Text dimColor>{item}</Text>
            )}
            {!isLast && <Text dimColor> &gt; </Text>}
          </Text>
        );
      })}
      {forwardItems.map((item, i) => (
        <Text key={`fwd-${i}`}>
          <Text dimColor> &gt; </Text>
          <Text dimColor strikethrough>{item}</Text>
        </Text>
      ))}
    </Text>
  );
}
