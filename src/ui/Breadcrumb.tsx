import React from "react";
import { Text } from "ink";

interface Props {
  items: string[];
}

export function Breadcrumb({ items }: Props) {
  if (items.length <= 1) return null;

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
    </Text>
  );
}
