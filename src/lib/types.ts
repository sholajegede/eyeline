export interface ScriptMeta {
  id: string;
  title: string;
  modified: number;
}

export interface Word {
  text: string;
  norm: string;
  spoken: boolean;
  cue: number;
  paragraph_start: boolean;
}

export interface Cue {
  title: string;
  start: number;
}

export interface ParsedScript {
  words: Word[];
  cues: Cue[];
}

export type PrompterMode = "notch" | "floating";

export interface Settings {
  scripts_dir: string | null;
  mode: PrompterMode;
  font_size: number;
  speed: number;
  mirror_x: boolean;
  mirror_y: boolean;
  text_color: string;
  width: number;
  height: number;
  hide_from_capture: boolean;
  countdown_secs: number;
  dim_read: boolean;
}

export type PrompterAction =
  | "toggle"
  | "faster"
  | "slower"
  | "next-cue"
  | "previous-cue";
