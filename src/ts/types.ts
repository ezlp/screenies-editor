/**
 * Shared types crossing the TS ↔ Rust bridge.
 *
 * ⚠ RULE: every interface here MUST mirror a #[derive(Serialize)] struct in
 * src-tauri/src/chatlog/mod.rs (serde is configured with rename_all = "camelCase").
 * If you change one side, change the other in the same commit.
 */

/** One colored run of text inside a line. */
export interface ColorSpan {
  text: string;
  /** Hex color like "#FFFFFF". */
  color: string;
  /** Heavier weight — used for system-tag prefixes like "VEHICLE: ". */
  bold: boolean;
}

/** Kind of chat line — drives auto-coloring and the M2 "RP only" filter. */
export type LineType = "normal" | "me" | "do" | "ooc" | "says" | "shouts" | "system";

/** A fully parsed chatlog line, ready to draw. */
export interface ParsedLine {
  spans: ColorSpan[];
  lineType: LineType;
}

/** Parsing rules — mirrors src-tauri/src/chatlog/preset.rs (serde camelCase). */
export interface ParsePreset {
  name: string;
  stripTimestamps: boolean;
  hexCodes: boolean;
  mePrefix: boolean;
  oocWrap: boolean;
  doSuffix: boolean;
  systemTags: boolean;
  radioChannels: string[];
  colorMe: string;
  colorOoc: string;
  colorDefault: string;
}
