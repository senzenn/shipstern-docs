'use client';

import { useTheme } from 'next-themes';
import { useCallback, useEffect, useId, useRef, useState } from 'react';

// Labels render at 14px. Do not shrink them below 12px to fit the column.
const minScale = 12 / 14;
// A layout change can toggle the page scrollbar. Undo a change only with this much room, so the two cannot loop.
const hysteresis = 32;

function viewBoxWidth(svg: string) {
  return Number(svg.match(/viewBox="[\d.-]+ [\d.-]+ ([\d.]+)/)?.[1] ?? 0);
}

export function Mermaid({ chart }: { chart: string }) {
  const id = useId().replaceAll(':', '');
  const { resolvedTheme } = useTheme();
  const containerRef = useRef<HTMLDivElement>(null);
  const horizontalWidth = useRef(0);
  const [vertical, setVertical] = useState(false);
  const [svg, setSvg] = useState<string>();
  const [scrollable, setScrollable] = useState(false);

  const source = chart.replaceAll('\\n', '\n');
  // A wide left-to-right flowchart turns top-to-bottom, so no label is clipped or too small.
  const flippable = /^\s*flowchart LR\b/.test(source);

  const fits = useCallback((width: number, margin = 0) => {
    const container = containerRef.current;
    if (!container) return true;
    const { paddingLeft, paddingRight } = getComputedStyle(container);
    const available = container.clientWidth - parseFloat(paddingLeft) - parseFloat(paddingRight);
    return width * minScale <= available - margin;
  }, []);

  useEffect(() => {
    let cancelled = false;

    void (async () => {
      const { default: mermaid } = await import('mermaid');
      await document.fonts.ready;
      if (cancelled) return;
      const dark = resolvedTheme === 'dark';

      mermaid.initialize({
        startOnLoad: false,
        securityLevel: 'strict',
        fontFamily: getComputedStyle(document.body).fontFamily,
        // SVG labels avoid inherited prose styles that can clip HTML labels.
        htmlLabels: false,
        layout: 'dagre',
        flowchart: { diagramPadding: 24, rankSpacing: 48, nodeSpacing: 32, wrappingWidth: 140 },
        sequence: { diagramMarginX: 8, actorMargin: 32, width: 150, wrap: true },
        theme: 'base',
        themeVariables: {
          darkMode: dark,
          background: 'transparent',
          primaryColor: dark ? '#121314' : '#f4f6f8',
          primaryTextColor: dark ? '#ededed' : '#0b0d0f',
          primaryBorderColor: dark ? '#c7f284' : '#007fa3',
          lineColor: dark ? '#9a9fa6' : '#5b6470',
          secondaryColor: dark ? '#0c0d0e' : '#ffffff',
          tertiaryColor: dark ? '#0c0d0e' : '#ffffff',
          clusterBkg: dark ? '#0c0d0e' : '#f8fafb',
          clusterBorder: dark ? '#2a2c2e' : '#d9dee3',
          noteBkgColor: dark ? '#151617' : '#eef2f5',
          noteBorderColor: dark ? '#2a2c2e' : '#d9dee3',
          noteTextColor: dark ? '#ededed' : '#0b0d0f',
          actorBkg: dark ? '#121314' : '#f4f6f8',
          actorBorder: dark ? '#c7f284' : '#007fa3',
          actorTextColor: dark ? '#ededed' : '#0b0d0f',
          signalColor: dark ? '#9a9fa6' : '#5b6470',
          signalTextColor: dark ? '#ededed' : '#0b0d0f',
          fontSize: '14px',
        },
      });

      const text = vertical ? source.replace(/flowchart LR\b/, 'flowchart TB') : source;
      const { svg } = await mermaid.render(`mermaid-${id}`, text);
      if (cancelled) return;
      if (!vertical) {
        horizontalWidth.current = viewBoxWidth(svg);
        if (flippable && !fits(horizontalWidth.current)) {
          setVertical(true);
          return;
        }
      }
      setScrollable(!fits(viewBoxWidth(svg)));
      setSvg(svg);
    })();

    return () => {
      cancelled = true;
    };
  }, [source, flippable, vertical, fits, id, resolvedTheme]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container || !svg) return;
    const observer = new ResizeObserver(() => {
      if (flippable && !vertical && !fits(horizontalWidth.current)) {
        setVertical(true);
        return;
      }
      if (flippable && vertical && fits(horizontalWidth.current, hysteresis)) {
        setVertical(false);
        return;
      }
      const width = viewBoxWidth(svg);
      setScrollable((was) => (was ? !fits(width, hysteresis) : !fits(width)));
    });
    observer.observe(container);
    return () => observer.disconnect();
  }, [svg, vertical, flippable, fits]);

  return (
    <figure className="not-prose my-6">
      <div
        ref={containerRef}
        className="overflow-x-auto rounded-xl border border-fd-border bg-fd-card p-2 focus-visible:outline-2 focus-visible:outline-fd-ring"
        role="region"
        aria-label="Diagram"
        tabIndex={scrollable ? 0 : undefined}
      >
        <div
          className="flex justify-center [&_svg]:h-auto [&_svg]:shrink-0"
          // The SVG scales down to minScale; past that, the box scrolls at full size.
          style={{ width: scrollable && svg ? `${Math.ceil(viewBoxWidth(svg))}px` : '100%', minWidth: '100%' }}
          // Mermaid output comes from the page's own diagram source.
          dangerouslySetInnerHTML={svg ? { __html: svg } : undefined}
        />
      </div>
      {scrollable && (
        <figcaption className="mt-2 text-xs text-fd-muted-foreground">
          Scroll horizontally to view the full diagram.
        </figcaption>
      )}
    </figure>
  );
}
