# System Architecture Diagrams

These diagrams are inline Graphviz DOT blocks. The mdbook Graphviz
preprocessor renders fenced `dot` blocks marked with `process`.

## Layered architecture (logical)

```dot process
digraph LayeredArchitecture {
  rankdir=TB;
  node [shape=box, style="rounded,filled", fillcolor="#f5f5f5"];
  interfaces [label="ui interfaces\ncli formatting / tui"];
  presentation [label="ui-presentation"];
  application [label="application"];
  domain [label="domain"];
  ports [label="ports"];
  adapters [label="adapters"];

  interfaces -> presentation -> application -> domain;
  application -> ports;
  adapters -> ports;
  adapters -> domain;
}
```

## Runtime component layout (physical)

```dot process
digraph RuntimeComponents {
  rankdir=TB;
  node [shape=box, style="rounded,filled", fillcolor="#f5f5f5"];

  subgraph cluster_interfaces {
    label="ui interfaces";
    style="rounded";
    terminal [label="ui/rotappo-ui-terminal\nformat output\ncli dispatch"];
    ui [label="ui/rotappo-ui-tui\nrender panels\nhandle input"];
  }

  presentation [label="ui/rotappo-ui-presentation\nformatting\nlogging cfg"];
  application [label="core/rotappo-application\nruntime loop"];
  ports [label="core/phenome-ports\nAssemblyPort\nHealthPort\nLogPort"];
  adapters [label="core/phenome-adapter-primer"];

  terminal -> presentation;
  terminal -> adapters;
  ui -> presentation;
  presentation -> application;
  application -> ports;
  adapters -> ports;
}
```

## Port boundaries

```dot process
digraph PortBoundaries {
  rankdir=LR;
  node [shape=box, style="rounded,filled", fillcolor="#f5f5f5"];

  interfaces -> presentation -> application -> ports;
  adapters -> ports;
  adapters -> domain;
  application -> domain;
}
```

Ports are the only surface the application layer can use to access
external data. Adapters implement those ports and normalize upstream
models into domain types.
