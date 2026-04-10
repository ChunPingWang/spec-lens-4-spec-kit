/**
 * TypeScript mirrors of the Rust IPC types defined in
 * `specs/001-speclens-desktop/contracts/ipc.md`. Keep in sync whenever
 * `src-tauri/src/models/*.rs` changes.
 */

export type Uuid = string;

export type LanguagePreference = "en" | "zh-TW" | "system";
export type ThemePreference = "light" | "dark" | "system";

export interface RecentProject {
  id: Uuid;
  name: string;
  path: string;
  lastOpenedAt: string;
  pinned: boolean;
}

export interface WindowBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface WindowHandle {
  windowId: string;
  projectId?: Uuid;
  bounds: WindowBounds;
}

export interface AppConfig {
  schemaVersion: number;
  language: LanguagePreference;
  theme: ThemePreference;
  recentProjects: RecentProject[];
  terminalBufferMaxLines: number;
  terminalDiskCapMiB: number;
  openWindows: WindowHandle[];
  notificationsEnabled: boolean;
  lastUpdatedAt: string;
}

export type StepStatus =
  | "not_started"
  | "in_progress"
  | "done"
  | "modified"
  | "missing"
  | "unknown";

export type DocKind =
  | "spec"
  | "plan"
  | "research"
  | "dataModel"
  | "contract"
  | "quickstart"
  | "tasks"
  | "other";

export type DocStatus = "generated" | "modified" | "missing" | "unverified";

export interface PhaseDocument {
  kind: DocKind;
  relativePath: string;
  status: DocStatus;
  sizeBytes: number;
  lastModifiedAt: string;
}

export interface Step {
  id: string;
  order: number;
  displayName: string;
  status: StepStatus;
  phaseDocuments: PhaseDocument[];
  taskFilePaths: string[];
  lastObservedAt?: string;
}

export type TaskFileFormat = "md" | "json" | "yaml" | "txt";

export interface TaskFile {
  relativePath: string;
  format: TaskFileFormat;
  displayName: string;
  taskCount: number;
  completedCount: number;
  parsedAt: string;
}

export type TaskStatus = "todo" | "in_progress" | "done" | "skipped";

export interface TaskEntry {
  id: string;
  title: string;
  status: TaskStatus;
  sectionPath: string[];
  filePath: string;
  line: number;
  raw: string;
}

export type EnvBadgeColor = "green" | "amber" | "red";

export interface EnvIssue {
  kind: "missing_binary" | "version_mismatch" | "path_not_writable" | "other";
  severity: "warn" | "error";
  message: string;
  hint?: string;
}

export interface EnvironmentStatus {
  speckitInstalled: boolean;
  speckitVersion?: string;
  speckitPath?: string;
  latestKnownVersion?: string;
  updateAvailable: boolean;
  checkedAt: string;
  issues: EnvIssue[];
}

export type AgentId =
  | "claude-code"
  | "copilot"
  | "gemini-cli"
  | "cursor"
  | "windsurf"
  | "amazon-q"
  | "codex-cli"
  | "qwen-code"
  | "opencode"
  | "kilo-code"
  | "auggie-cli"
  | "roo-code"
  | "generic";

export type AgentDetectionSource = "project" | "path" | "none";

export interface AgentProfile {
  id: AgentId;
  displayName: string;
  iconKey: string;
  detectedFrom: AgentDetectionSource;
  version?: string;
  highlightRuleSet: string;
}

export type PhaseTab = "overview" | "documents" | "tasks";

export interface ProjectState {
  schemaVersion: number;
  lastStepId?: string;
  lastPhaseTab?: PhaseTab;
  selectedTaskFilePaths: string[];
  activeTaskFilePath?: string;
  terminalCollapsed: boolean;
  lastOpenedAt: string;
}

export interface Project {
  id: Uuid;
  name: string;
  rootPath: string;
  speckitDetected: boolean;
  openedAt: string;
  environment: EnvironmentStatus;
  agent: AgentProfile;
  steps: Step[];
  taskFiles: TaskFile[];
  state: ProjectState;
}

export type OutputStream = "stdout" | "stderr";

export interface AnsiSpan {
  start: number;
  end: number;
  fg?: number;
  bg?: number;
  bold: boolean;
  italic: boolean;
  underline: boolean;
}

export type HighlightSeverity = "info" | "success" | "warn" | "error";

export interface HighlightMatch {
  ruleId: string;
  severity: HighlightSeverity;
}

export interface OutputLine {
  seq: number;
  ts: number;
  stream: OutputStream;
  text: string;
  ansiSpans?: AnsiSpan[];
  highlight?: HighlightMatch;
  stepId?: string;
}

export interface PtySessionDescriptor {
  sessionId: Uuid;
  windowId: string;
  projectId: Uuid;
  cwd: string;
  shell: string;
  startedAt: string;
}

export interface IpcError {
  code: string;
  message: string;
  hint?: string;
}

export function isIpcError(value: unknown): value is IpcError {
  return (
    typeof value === "object" &&
    value !== null &&
    typeof (value as IpcError).code === "string" &&
    typeof (value as IpcError).message === "string"
  );
}
