# TECH.md — SpecLens
## 技術需求文件 (Technical Requirements Document)

**版本**: v1.3.0  
**日期**: 2026-04-10  
**狀態**: Draft  
**產品名稱**: SpecLens  
**架構模式**: Tauri v2 (Rust Backend + React Frontend)

---

## 1. 技術棧決策

### 1.1 框架選型：Tauri v2

| 考量點 | Tauri v2 | Electron |
|-------|---------|----------|
| 後端語言 | ✅ Rust 原生 | ❌ Node.js |
| 安裝包大小 | ✅ ~8MB | ❌ ~100MB |
| 記憶體佔用 | ✅ < 50MB | ❌ > 200MB |
| 跨平台 | ✅ Mac/Win/Linux | ✅ |
| 安全沙盒 | ✅ 雙層模型 | ⚠️ 較寬鬆 |

### 1.2 前端選型：React + TypeScript + Vite

- UI：`shadcn/ui` + `Tailwind CSS`
- Terminal：`@xterm/xterm` v5
- 虛擬列表：`@tanstack/react-virtual`
- 狀態管理：`Zustand`
- Markdown 渲染：`marked` + `highlight.js`
- 圖示：`lucide-react`

---

## 2. 系統架構

```
┌──────────────────────────────────────────────────────┐
│                     SpecLens                           │
│                                                       │
│  ┌────────────────────────────────────────────────┐  │
│  │           Frontend (React + TS)                 │  │
│  │  TopBar(AgentBadge) │ StepsList         │  │
│  │  PhaseTabs → [Overview][Documents][Tasks]       │  │
│  │  TaskFileBar │ TaskFilePicker                   │  │
│  │  TerminalPanel(xterm.js + HighlightRules)       │  │
│  └─────────────────┬──────────────────────────────┘  │
│                    │ Tauri IPC                        │
│  ┌─────────────────▼──────────────────────────────┐  │
│  │             Backend (Rust)                      │  │
│  │  speckit_engine │ terminal_bridge(PTY+Buffer)   │  │
│  │  phase_scanner  │ task_file_scanner             │  │
│  │  tasks_parser   │ agent_detector                │  │
│  │  doc_hash_store │ fs_watcher                    │  │
│  └────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────┘
```

---

## 3. 後端設計 (Rust Backend)

### 3.1 Cargo 依賴

```toml
[dependencies]
tauri            = { version = "2", features = ["shell-open", "notification"] }
tauri-plugin-fs  = "2"
tauri-plugin-shell  = "2"
tauri-plugin-notification = "2"
tauri-plugin-store = "2"
tokio            = { version = "1", features = ["full"] }
serde            = { version = "1", features = ["derive"] }
serde_json       = "1"
serde_yaml       = "0.9"
reqwest          = { version = "0.12", features = ["json", "rustls-tls"] }
anyhow           = "1"
thiserror        = "1"
portable-pty     = "0.8"
bytes            = "1"
notify           = "6"
semver           = "1"
sha2             = "0.10"    # SHA-256 hash for DocStatus
hex              = "0.4"
uuid             = { version = "1", features = ["v4"] }
chrono           = { version = "0.4", features = ["serde"] }
tracing          = "0.1"
tracing-subscriber = "0.3"

[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["wincred"] }
```

### 3.2 模組結構

```
src-tauri/src/
├── main.rs
├── lib.rs
├── commands/
│   ├── project.rs        # open_project, get_recent_projects
│   ├── speckit.rs        # check_environment, install, update, get_steps
│   ├── terminal.rs       # start_pty, write_to_pty, kill_pty, get_output_slice
│   ├── phase.rs          # get_phase_outputs, get_document_preview
│   ├── tasks.rs          # scan_task_files, get_tasks_from_path, build_task_file_entry
│   ├── agent.rs          # detect_project_agent
│   └── config.rs
├── services/
│   ├── speckit_engine.rs
│   ├── terminal_bridge.rs   # PTY + 時間戳輸出緩衝
│   ├── version_checker.rs
│   ├── phase_scanner.rs     # 掃描 output_dir，含 DocStatus hash 邏輯
│   ├── task_file_scanner.rs # 掃描候選任務檔
│   ├── tasks_parser.rs      # 解析任意路徑任務檔（md/json/yaml）
│   ├── agent_detector.rs    # 12 Agent 特徵偵測
│   ├── doc_hash_store.rs    # SHA-256 hash 持久化
│   └── fs_watcher.rs
├── models/
│   ├── spec_step.rs      # SpecStep, StepStatus
│   ├── phase_output.rs   # PhaseOutput, OutputDocument, DocStatus
│   ├── task.rs           # Task, TaskStatus, TasksProgress
│   ├── task_file.rs      # TaskFileEntry, TaskFileBar, AggregateProgress
│   ├── agent.rs          # AgentType, AgentInfo, DetectionSource
│   ├── terminal.rs       # OutputLine { data, timestamp_ms }
│   └── environment.rs
└── error.rs
```

### 3.3 核心資料模型

```rust
// models/spec_step.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecStep {
    pub id: String,
    pub index: u32,
    pub name: String,
    pub description: String,
    pub step_type: StepType,
    pub status: StepStatus,
    pub dependencies: Vec<String>,
    pub started_at: Option<DateTime<Utc>>,    // running 開始時間
    pub completed_at: Option<DateTime<Utc>>,  // completed/failed 時間
    pub duration_ms: Option<u64>,
    pub output_summary: Option<String>,
}

// models/terminal.rs — 每行輸出附帶時間戳（用於步驟關聯）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputLine {
    pub data: String,
    pub timestamp_ms: i64,     // Unix timestamp in milliseconds
}

// models/phase_output.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DocStatus {
    Generated,    // hash 與首次記錄一致
    Modified,     // hash 與首次記錄不同
    Missing,
    Unverified,   // 冷啟動首次掃描，無歷史記錄
}

// models/task_file.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFileEntry {
    pub id: String,
    pub file_name: String,
    pub display_name: String,      // "001-auth/tasks.md"
    pub absolute_path: String,
    pub relative_path: String,
    pub format: TaskFileFormat,
    pub progress: Option<TasksProgress>,
    pub is_pinned: bool,
}

// models/agent.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    ClaudeCode, GitHubCopilot, GeminiCli, Cursor, Windsurf,
    AmazonQ, CodexCli, QwenCode, Opencode, KiloCode, AuggieCli, RooCode,
    Unknown,
}
```

### 3.4 Terminal Bridge：時間戳輸出緩衝

```rust
// services/terminal_bridge.rs

/// 環狀輸出緩衝：保留最近 50,000 行，每行附 timestamp_ms
pub struct OutputBuffer {
    lines: VecDeque<OutputLine>,
    max_lines: usize,
}

impl OutputBuffer {
    pub fn push(&mut self, data: String) {
        let line = OutputLine {
            data,
            timestamp_ms: Utc::now().timestamp_millis(),
        };
        if self.lines.len() >= self.max_lines {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    /// 查詢時間區間內的輸出片段（用於步驟關聯）
    pub fn slice(&self, start_ms: i64, end_ms: i64) -> Vec<OutputLine> {
        self.lines.iter()
            .filter(|l| l.timestamp_ms >= start_ms && l.timestamp_ms <= end_ms)
            .cloned()
            .collect()
    }

    /// Task 近似片段：完成時間戳往前 30 秒
    pub fn slice_before(&self, completed_ms: i64, seconds: i64) -> Vec<OutputLine> {
        self.slice(completed_ms - seconds * 1000, completed_ms)
    }
}

// IPC Command：查詢步驟對應的輸出片段
#[tauri::command]
pub async fn get_output_slice(
    start_ms: i64,
    end_ms: i64,
    state: State<'_, AppState>,
) -> Result<Vec<OutputLine>, String> {
    let buffer = state.output_buffer.lock().await;
    Ok(buffer.slice(start_ms, end_ms))
}
```

### 3.5 DocStatus：FS Watcher Hash 判斷

```rust
// services/doc_hash_store.rs

/// 記錄每個檔案首次 Create 事件時的 SHA-256 hash
/// 儲存於 workspace.json，跨會話持久化
pub struct DocHashStore {
    hashes: HashMap<String, String>,  // absolute_path → hex hash
}

impl DocHashStore {
    pub fn record_generated(&mut self, path: &str) -> Result<()> {
        let hash = sha256_file(path)?;
        self.hashes.insert(path.to_string(), hash);
        Ok(())
    }

    pub fn resolve_status(&self, path: &str) -> DocStatus {
        match self.hashes.get(path) {
            None => DocStatus::Unverified,   // 冷啟動，無歷史記錄
            Some(original_hash) => {
                let current_hash = sha256_file(path).unwrap_or_default();
                if current_hash == *original_hash {
                    DocStatus::Generated
                } else {
                    DocStatus::Modified
                }
            }
        }
    }
}

// FS Watcher 整合：Create → 記錄 hash；Modify → 比對更新 DocStatus
pub fn watch_phase_outputs(
    output_dir: PathBuf,
    app: AppHandle,
    hash_store: Arc<Mutex<DocHashStore>>,
) -> Result<RecommendedWatcher> {
    let mut watcher = notify::recommended_watcher(move |res| {
        match res {
            Ok(Event { kind: EventKind::Create(_), paths, .. }) => {
                for path in &paths {
                    let mut store = hash_store.blocking_lock();
                    store.record_generated(&path.to_string_lossy()).ok();
                }
                app.emit("phase_documents_changed", ()).ok();
            }
            Ok(Event { kind: EventKind::Modify(_), paths, .. }) => {
                // hash 比對在 get_phase_outputs 呼叫時即時執行
                app.emit("phase_documents_changed", ()).ok();
            }
            _ => {}
        }
    })?;
    watcher.watch(&output_dir, RecursiveMode::NonRecursive)?;
    Ok(watcher)
}
```

### 3.6 Tasks Parser：支援任意路徑

```rust
// services/tasks_parser.rs
// 統一入口：接受任意絕對路徑，格式由副檔名自動識別
// 注意：get_tasks_progress（舊版硬編碼路徑）已移除，統一使用此入口

pub async fn parse_tasks_from_path(path: &Path) -> Result<TasksProgress> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("md") | Some("markdown") => parse_tasks_markdown(path).await,
        Some("json")                  => parse_tasks_json(path).await,
        Some("yaml") | Some("yml")    => parse_tasks_yaml(path).await,
        _                             => parse_tasks_plaintext(path).await,
    }
}

/// GFM Checklist 解析："- [ ]" 未完成，"- [x]" 已完成
async fn parse_tasks_markdown(path: &Path) -> Result<TasksProgress> {
    let content = tokio::fs::read_to_string(path).await?;
    let mut tasks = Vec::new();
    let mut index = 0u32;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
            tasks.push(Task { index, title: rest.trim().to_string(),
                status: TaskStatus::Todo, ..Task::default() });
            index += 1;
        } else if let Some(rest) = trimmed.strip_prefix("- [x] ")
            .or_else(|| trimmed.strip_prefix("- [X] "))
        {
            tasks.push(Task { index, title: rest.trim().to_string(),
                status: TaskStatus::Done, ..Task::default() });
            index += 1;
        }
    }

    let completed = tasks.iter().filter(|t| t.status == TaskStatus::Done).count() as u32;
    let total = tasks.len() as u32;
    Ok(TasksProgress {
        total, completed,
        percent: if total > 0 { completed as f32 / total as f32 * 100.0 } else { 0.0 },
        tasks,
    })
}

#[tauri::command]
pub async fn get_tasks_from_path(absolute_path: String) -> Result<TasksProgress, String> {
    parse_tasks_from_path(Path::new(&absolute_path))
        .await.map_err(|e| e.to_string())
}
```

### 3.7 Agent Detector

```rust
// services/agent_detector.rs

const AGENT_SIGNATURES: &[(&str, AgentType, &str)] = &[
    (".claude",                         AgentType::ClaudeCode,    "Claude Code"),
    ("CLAUDE.md",                       AgentType::ClaudeCode,    "Claude Code"),
    (".github/copilot-instructions.md", AgentType::GitHubCopilot, "GitHub Copilot"),
    ("GEMINI.md",                       AgentType::GeminiCli,     "Gemini CLI"),
    (".gemini",                         AgentType::GeminiCli,     "Gemini CLI"),
    (".cursor",                         AgentType::Cursor,        "Cursor"),
    (".cursorrules",                    AgentType::Cursor,        "Cursor"),
    (".windsurfrules",                  AgentType::Windsurf,      "Windsurf"),
    (".amazonq",                        AgentType::AmazonQ,       "Amazon Q Developer"),
    (".qwen",                           AgentType::QwenCode,      "Qwen Code"),
    ("opencode.json",                   AgentType::Opencode,      "opencode"),
    (".kilocode",                       AgentType::KiloCode,      "Kilo Code"),
    (".augment",                        AgentType::AuggieCli,     "Auggie CLI"),
    (".roo",                            AgentType::RooCode,       "Roo Code"),
    (".roosettings",                    AgentType::RooCode,       "Roo Code"),
];

pub async fn detect_agent(project_path: &Path) -> Result<AgentInfo> {
    // 優先讀取 specify.yaml 的 --ai 記錄
    if let Some(info) = read_speckit_config_agent(project_path).await? {
        return Ok(info);
    }
    // 掃描特徵檔案
    for (sig, agent_type, display_name) in AGENT_SIGNATURES {
        if project_path.join(sig).exists() {
            return Ok(AgentInfo {
                agent_type: agent_type.clone(),
                display_name: display_name.to_string(),
                icon_key: agent_type_to_icon_key(agent_type),
                detection_source: DetectionSource::FileSignature,
                version: None,
            });
        }
    }
    // 通用模式
    Ok(AgentInfo {
        agent_type: AgentType::Unknown,
        display_name: "通用模式".to_string(),
        icon_key: "generic".to_string(),
        detection_source: DetectionSource::NotDetected,
        version: None,
    })
}
```

### 3.8 Universal Highlight Rules（通用視覺高亮）

```rust
// services/highlight_rules.rs
// 不做語意解析，純 regex 模式比對，對所有 Agent 一致套用

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightRule {
    pub name: String,
    pub pattern: String,         // regex
    pub css_class: String,       // 對應前端 CSS class
    pub enabled: bool,
}

pub fn default_rules() -> Vec<HighlightRule> {
    vec![
        HighlightRule { name: "success".into(),
            pattern: r"^(✓|✅|✔|\[done\]|\[ok\])".into(),
            css_class: "hl-success".into(), enabled: true },
        HighlightRule { name: "error".into(),
            pattern: r"^(✗|❌|Error|Failed|FAILED)".into(),
            css_class: "hl-error".into(), enabled: true },
        HighlightRule { name: "progress".into(),
            pattern: r"^(→|▶|Running|\[\.{3}\])".into(),
            css_class: "hl-progress".into(), enabled: true },
        HighlightRule { name: "warning".into(),
            pattern: r"^(⚠|Warning|WARN)".into(),
            css_class: "hl-warning".into(), enabled: true },
        HighlightRule { name: "filepath".into(),
            pattern: r"(\.\/|\/)[^\s]+".into(),
            css_class: "hl-filepath".into(), enabled: true },
    ]
}
// 規則儲存於 workspace.json，使用者可自訂新增或停用
```

### 3.9 資產管理

```
src-tauri/
└── icons/
```

**Tauri asset protocol 載入（前端）**：
```typescript
import { convertFileSrc } from '@tauri-apps/api/tauri';
import { resourceDir } from '@tauri-apps/api/path';

## 4. 前端設計 (React Frontend)

### 4.1 目錄結構

```
src/
├── main.tsx
├── App.tsx
├── pages/
│   ├── Welcome.tsx              # SpecLens Logo + 最近專案
│   └── Workspace.tsx
├── components/
│   ├── layout/
│   │   ├── AppShell.tsx
│   │   ├── TopBar.tsx           # AgentBadge + SpecLens名稱 + 進度
│   │   └── ResizablePanel.tsx
│   ├── steps/
│   │   ├── StepsList.tsx
│   │   ├── StepItem.tsx
│   │   └── ProgressBar.tsx
│   ├── phase/
│   │   ├── PhaseTabs.tsx        # Phase Tab 列（步驟同步）
│   │   ├── PhaseTabPanel.tsx    # 三層子頁容器
│   │   ├── OverviewPanel.tsx    # Overview 子頁（步驟詳情 + 輸出關聯）
│   │   ├── DocumentList.tsx
│   │   ├── DocumentItem.tsx
│   │   └── DocumentPreview.tsx
│   ├── tasks/
│   │   ├── TasksPanel.tsx
│   │   ├── TaskFileBar.tsx      # 永遠顯示，1個Tab時隱藏箭頭與關閉
│   │   ├── TaskFilePicker.tsx   # 首次進入必顯示，絕不自動載入
│   │   ├── TasksProgressPanel.tsx  # 多檔雙列 / 單檔單列
│   │   ├── TaskFilter.tsx
│   │   ├── TaskItem.tsx
│   │   └── TaskDetail.tsx       # 含Terminal近似片段（≈完成前30秒）
│   ├── terminal/
│   │   ├── TerminalPanel.tsx    # xterm.js + 通用高亮規則
│   │   ├── TerminalToolbar.tsx
│   │   └── QuickCommands.tsx
│   ├── agent/
│   │   ├── AgentBadge.tsx
│   │   └── AgentIcon.tsx
│   ├── brand/
│   └── env/
│       ├── EnvCheckDialog.tsx
│       ├── InstallGuide.tsx
│       └── UpdateBanner.tsx
├── stores/
│   ├── projectStore.ts
│   ├── stepsStore.ts
│   ├── terminalStore.ts         # outputLines: OutputLine[] (含timestamp_ms)
│   ├── phaseStore.ts
│   ├── tasksStore.ts
│   ├── taskFileBarStore.ts      # entries 永遠顯示；1個時隱藏箭頭
│   └── agentStore.ts
├── hooks/
│   ├── useSpeckitEvents.ts
│   ├── usePtySession.ts
│   ├── useEnvCheck.ts
│   ├── usePhaseSync.ts
│   ├── useDocumentPreview.ts
│   └── useOutputSlice.ts        # 查詢步驟/Task 對應輸出片段
├── lib/
│   ├── tauri.ts
│   ├── markdown.ts
│   ├── highlightRules.ts        # 通用高亮 regex 套用
│   └── utils.ts
└── types/
    ├── speckit.ts
    ├── phase.ts
    ├── tasks.ts
    ├── taskFile.ts
    ├── agent.ts
    └── terminal.ts              # OutputLine { data, timestamp_ms }
```

### 4.2 Zustand Stores

```typescript
// stores/stepsStore.ts
interface StepsState {
  steps: SpecStep[];
  currentStepId: string | null;
  overallProgress: number;
  setSteps: (steps: SpecStep[]) => void;
  setCurrentStep: (id: string | null) => void;
  refreshFromBackend: (projectPath: string) => Promise<void>;
}

// stores/terminalStore.ts
interface TerminalState {
  sessionId: string | null;
  isConnected: boolean;
  filterText: string;
  isAutoScroll: boolean;
  outputLines: OutputLine[];   // 帶 timestamp_ms
  appendOutput: (line: OutputLine) => void;
  clearOutput: () => void;
}

// stores/taskFileBarStore.ts
interface TaskFileBarState {
  entries: TaskFileEntry[];         // 1+ 個時永遠顯示 Bar
  activeEntryId: string | null;
  isPickerOpen: boolean;            // 首次進入 tasks Tab 時設為 true
  aggregateProgress: AggregateProgress;
  addEntry: (entry: TaskFileEntry) => void;
  removeEntry: (id: string) => void;
  setActiveEntry: (id: string) => void;
  openPicker: () => void;
  closePicker: () => void;
  persistToStore: (projectPath: string) => Promise<void>;
  loadFromStore: (projectPath: string) => Promise<void>;
}

// stores/agentStore.ts
interface AgentState {
  agentInfo: AgentInfo | null;
  isDetecting: boolean;
  detectAgent: (projectPath: string) => Promise<void>;
}
```

### 4.4 OverviewPanel：步驟詳情 + 輸出關聯

```tsx
// components/phase/OverviewPanel.tsx

export function OverviewPanel({ step }: { step: SpecStep }) {
  const [outputSlice, setOutputSlice] = useState<OutputLine[]>([]);
  const [isOutputOpen, setIsOutputOpen] = useState(false);

  // 載入步驟對應的 Terminal 輸出片段
  useEffect(() => {
    if (!step.started_at || !step.completed_at) return;
    const startMs = new Date(step.started_at).getTime();
    const endMs   = new Date(step.completed_at).getTime();
    invoke<OutputLine[]>('get_output_slice', { startMs, endMs })
      .then(setOutputSlice);
  }, [step.id, step.completed_at]);

  return (
    <div className="p-4 space-y-4">
      {/* 步驟基本資訊 */}
      <div className="grid grid-cols-2 gap-3 text-sm">
        <InfoRow label="狀態">   <StepStatusBadge status={step.status} /></InfoRow>
        <InfoRow label="執行時長">{step.duration_ms ? formatDuration(step.duration_ms) : '—'}</InfoRow>
        <InfoRow label="啟動時間">{step.started_at   ? formatTime(step.started_at)   : '—'}</InfoRow>
        <InfoRow label="完成時間">{step.completed_at ? formatTime(step.completed_at) : '—'}</InfoRow>
      </div>

      {/* 描述 */}
      {step.description && (
        <section>
          <h4 className="text-xs font-semibold text-muted-foreground mb-1">描述</h4>
          <p className="text-sm">{step.description}</p>
        </section>
      )}

      {/* Terminal 輸出關聯（可展開） */}
      {outputSlice.length > 0 && (
        <section>
          <button
            onClick={() => setIsOutputOpen(v => !v)}
            className="flex items-center gap-2 text-xs font-semibold text-muted-foreground"
          >
            <ChevronDown className={cn('h-3 w-3 transition-transform', isOutputOpen && 'rotate-180')} />
            Terminal 輸出片段（{outputSlice.length} 行）
          </button>
          {isOutputOpen && (
            <pre className="mt-2 text-xs bg-muted rounded p-2 overflow-auto max-h-40">
              {outputSlice.map((l, i) => (
                <div key={i} className="font-mono">{l.data}</div>
              ))}
            </pre>
          )}
          <button
            className="mt-1 text-xs text-primary hover:underline"
            onClick={() => { /* 捲動至 Terminal 面板並高亮此時間區間 */ }}
          >
            在 Terminal 面板查看完整輸出 →
          </button>
        </section>
      )}
    </div>
  );
}
```

### 4.5 TaskFileBar 元件（永遠顯示）

```tsx
// components/tasks/TaskFileBar.tsx
// 規則：
//   entries.length === 0 → 顯示「選擇任務檔」空狀態
//   entries.length === 1 → 顯示 Bar，隱藏捲動箭頭與關閉按鈕（×）
//   entries.length >= 2  → 完整 Bar，含捲動與關閉

export function TaskFileBar() {
  const { entries, activeEntryId, setActiveEntry, removeEntry, openPicker } =
    useTaskFileBarStore();

  const scrollRef = useRef<HTMLDivElement>(null);
  const [canScrollLeft, setCanScrollLeft]   = useState(false);
  const [canScrollRight, setCanScrollRight] = useState(false);
  const isSingle = entries.length === 1;

  // ... scroll detection logic ...

  if (entries.length === 0) {
    return (
      <div className="flex items-center gap-2 px-3 py-2 border-b border-border bg-muted/20">
        <span className="text-xs text-muted-foreground">尚未選擇任務檔</span>
        <button onClick={openPicker} className="text-xs text-primary hover:underline">
          + 選擇任務檔
        </button>
      </div>
    );
  }

  return (
    <div className="flex items-center border-b border-border bg-muted/10 select-none">
      {/* 捲動箭頭（單檔時隱藏）*/}
      {!isSingle && canScrollLeft && (
        <button onClick={() => scroll('left')} className="px-1">
          <ChevronLeft className="h-4 w-4" />
        </button>
      )}

      <div ref={scrollRef} onScroll={checkScroll}
           className="flex-1 flex overflow-x-auto scrollbar-none">
        {entries.map((entry) => {
          const isActive = entry.id === activeEntryId;
          return (
            <button key={entry.id} onClick={() => setActiveEntry(entry.id)}
              className={cn('group flex items-center gap-2 px-3 py-2 text-xs whitespace-nowrap border-b-2 transition-colors',
                isActive ? 'border-primary text-foreground' : 'border-transparent text-muted-foreground')}>
              <span className="max-w-[140px] truncate">{entry.display_name}</span>
              <ProgressBadge progress={entry.progress} />
              {/* 關閉按鈕（單檔時隱藏）*/}
              {!isSingle && (
                <span onClick={(e) => { e.stopPropagation(); removeEntry(entry.id); }}
                      className="hidden group-hover:flex h-3.5 w-3.5 items-center justify-center rounded-full hover:bg-muted-foreground/20">
                  <X className="h-2.5 w-2.5" />
                </span>
              )}
            </button>
          );
        })}
      </div>

      {!isSingle && canScrollRight && (
        <button onClick={() => scroll('right')} className="px-1">
          <ChevronRight className="h-4 w-4" />
        </button>
      )}

      <button onClick={openPicker} className="px-2 border-l border-border">
        <Plus className="h-3.5 w-3.5" />
      </button>
    </div>
  );
}
```

### 4.6 TaskFilePicker（首次必顯示，絕不自動載入）

```tsx
// components/tasks/TaskFilePicker.tsx
// 觸發時機：
//   1. 首次切換至 tasks Tab（taskFileBarStore.entries.length === 0）
//   2. 使用者點擊「⚙ 變更來源」或「+ 新增檔案」

export function TaskFilePicker({ projectPath }: { projectPath: string }) {
  const { isPickerOpen, closePicker, addEntry } = useTaskFileBarStore();
  const [candidates, setCandidates] = useState<TaskFileEntry[]>([]);
  const [selected, setSelected]     = useState<Set<string>>(new Set());
  const [isScanning, setIsScanning] = useState(false);

  useEffect(() => {
    if (!isPickerOpen) return;
    setIsScanning(true);
    // 掃描候選清單（僅供選擇，不自動勾選）
    invoke<TaskFileEntry[]>('scan_task_files', { projectPath })
      .then(setCandidates)
      .finally(() => setIsScanning(false));
  }, [isPickerOpen, projectPath]);

  const handleApply = async () => {
    for (const id of selected) {
      const entry = candidates.find(c => c.id === id);
      if (!entry) continue;
      const progress = await invoke<TasksProgress>('get_tasks_from_path', {
        absolutePath: entry.absolute_path,
      }).catch(() => null);
      addEntry({ ...entry, progress: progress ?? undefined });
    }
    closePicker();
  };

  // ... render Dialog with candidates list ...
}
```

### 4.7 通用高亮規則套用

```typescript
// lib/highlightRules.ts

export interface HighlightRule {
  name: string;
  pattern: RegExp;
  cssClass: string;
  enabled: boolean;
}

export const DEFAULT_RULES: HighlightRule[] = [
  { name: 'success',  pattern: /^(✓|✅|✔|\[done\]|\[ok\])/,     cssClass: 'hl-success',  enabled: true },
  { name: 'error',    pattern: /^(✗|❌|Error|Failed|FAILED)/,     cssClass: 'hl-error',    enabled: true },
  { name: 'progress', pattern: /^(→|▶|Running|\[\.{3}\])/,        cssClass: 'hl-progress', enabled: true },
  { name: 'warning',  pattern: /^(⚠|Warning|WARN)/,              cssClass: 'hl-warning',  enabled: true },
  { name: 'filepath', pattern: /(\.\/|\/)[^\s]+/g,                cssClass: 'hl-filepath', enabled: true },
];

export function applyHighlightRules(line: string, rules: HighlightRule[]): string {
  // 回傳帶有 <span class="hl-*"> 標記的 HTML 字串
  // xterm.js 的 CustomRenderer 套用這些 CSS class
  for (const rule of rules.filter(r => r.enabled)) {
    if (rule.pattern.test(line)) {
      return `<span class="${rule.cssClass}">${escapeHtml(line)}</span>`;
    }
  }
  return escapeHtml(line);
}
```

---

## 5. IPC 介面規格

### 5.1 Commands

| Command | 參數 | 返回值 | 說明 |
|---------|------|--------|------|
| `open_project` | `{ path }` | `ProjectInfo` | 開啟專案 |
| `get_recent_projects` | — | `ProjectInfo[]` | 最近專案 |
| `check_environment` | — | `EnvCheckResult` | 環境檢查 |
| `install_speckit` | — | `void` | 安裝 Spec-Kit |
| `update_speckit` | — | `void` | 更新 Spec-Kit |
| `get_all_steps` | `{ projectPath }` | `SpecStep[]` | 全部步驟 |
| `get_current_step` | `{ projectPath }` | `SpecStep\|null` | 當前步驟 |
| `get_phase_outputs` | `{ projectPath, phaseId }` | `PhaseOutput` | 階段產出文件 |
| `get_document_preview` | `{ absolutePath }` | `string` | 文件預覽（前 4KB）|
| `scan_task_files` | `{ projectPath }` | `TaskFileEntry[]` | 掃描候選任務檔 |
| `scan_task_files_in_dir` | `{ dirPath, projectPath }` | `TaskFileEntry[]` | 指定資料夾掃描 |
| `build_task_file_entry` | `{ absolutePath, projectPath }` | `TaskFileEntry` | 建立任務檔條目 |
| `get_tasks_from_path` | `{ absolutePath }` | `TasksProgress` | 解析任務檔（唯一入口）|
| `detect_project_agent` | `{ projectPath }` | `AgentInfo` | 偵測 AI Agent |
| `get_output_slice` | `{ startMs, endMs }` | `OutputLine[]` | 步驟對應輸出片段 |
| `start_pty_session` | `{ projectPath }` | `{ sessionId }` | 啟動 PTY |
| `write_to_pty` | `{ sessionId, data }` | `void` | 寫入 PTY |
| `resize_pty` | `{ sessionId, cols, rows }` | `void` | 調整 PTY 大小 |
| `kill_pty_session` | `{ sessionId }` | `void` | 終止 PTY |
| `export_terminal_log` | `{ content, path }` | `void` | 匯出日誌 |

### 5.2 Events

| Event | Payload | 觸發時機 |
|-------|---------|---------|
| `steps_state_changed` | `void` | speckit 狀態檔變更 |
| `pty_output` | `OutputLine` | PTY 新輸出（含 timestamp_ms）|
| `install_progress` | `{ percent, message }` | 安裝進度 |
| `env_check_complete` | `EnvCheckResult` | 環境檢查完成 |
| `step_status_updated` | `{ stepId, status, timestamp_ms }` | 步驟狀態變更（含時間戳）|
| `phase_documents_changed` | `{ phaseId }` | 階段文件新增/變更 |
| `task_file_changed` | `{ absolutePath }` | 任務檔內容變更 |
| `agent_detected` | `AgentInfo` | Agent 偵測完成 |

---

## 6. 資料持久化

```json
// app_data/settings.json（全域）
{
  "recentProjects": [...],
  "preferences": {
    "theme": "dark",
    "fontSize": 13,
    "terminalEnabled": true,
    "notificationsEnabled": true,
    "autoScrollTerminal": true,
    "highlightRules": []          // 使用者自訂規則（覆蓋預設）
  },
  "speckitPath": "/usr/local/bin/speckit"
}

// app_data/projects/<project-hash>/workspace.json（每專案獨立）
{
  "taskFileBar": {
    "entries": [
      {
        "id": "uuid-1",
        "displayName": "001-auth/tasks.md",
        "absolutePath": "...",
        "relativePath": ".agent/features/001-auth/tasks.md",
        "format": "markdown_checklist",
        "isPinned": true
      }
    ],
    "activeEntryId": "uuid-1"
  },
  "docHashes": {
    "/abs/path/to/PRD.md": "a3f2c1...",
    "/abs/path/to/TECH.md": "b9e4d2..."
  },
  "agentOverride": null,
  "lastActivePhaseId": "tasks",
  "lastActiveSubTab": "tasks"
}
```

**儲存位置**：

| 平台 | 路徑 |
|-----|------|
| macOS | `~/Library/Application Support/speclens/` |
| Windows | `%APPDATA%\speclens\` |
| Linux | `~/.config/speclens/` |

---

## 7. 安全性設定（雙層模型）

```json
// tauri.conf.json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; script-src 'self'; connect-src ipc: http://ipc.localhost",
      "assetProtocol": {
        "enable": true,
      }
    }
  },
  "plugins": {
    "fs": {
      "scope": {
        "allow": ["$HOME/**"],
        "deny":  ["$HOME/.ssh/**", "$HOME/.gnupg/**"]
      }
    },
    "shell": {
      "open": false,
      "scope": [
        {
          "name": "speckit",
          "cmd": "speckit",
          "args": true,
          "cwd": "$PROJECT_PATH"
        }
      ]
    }
  }
}
```

| 操作 | 限制 |
|------|------|
| PTY 執行 | `cwd` 強制鎖定專案目錄 |
| 檔案寫入 | 限制在專案目錄內 |
| 唯讀存取 | 允許 `$HOME/**`（由 OS 選擇器授權）|

---

## 8. 建置與打包

```bash
# 必要工具
node >= 20.0.0 | pnpm >= 9.0.0 | rust >= 1.78.0 (stable)
cargo install tauri-cli --version "^2"

pnpm tauri dev    # 開發
pnpm tauri build  # 生產建置
```

| 平台 | 產出格式 | 大小預估 |
|-----|---------|---------|
| macOS | `.dmg` + `.app` | ~8MB |
| Windows | `.msi` + `.exe` (NSIS) | ~10MB |
| Linux | `.deb` + `.AppImage` + `.rpm` | ~12MB |

---

## 9. 測試策略

**Rust 後端**：`#[tokio::test]` 單元測試覆蓋 `tasks_parser`、`agent_detector`、`doc_hash_store`、`output_buffer.slice()`。

**React 前端（Vitest）**：`TaskFileBar`（0 / 1 / 多條目三情境）、`TaskFilePicker`（首次強制顯示）、`OverviewPanel`（輸出片段展開）。

**E2E（Playwright + Tauri Driver）**：開啟專案 → 首次 tasks Tab → Picker 必出現 → 選擇任務檔 → Bar 正確顯示。

---

## 10. 專案結構

```
speclens/
├── src/                    # React 前端
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── icons/
│   │   ├── app/            # SpecLens 應用圖示
│   └── src/
├── public/
├── package.json
├── vite.config.ts
├── tsconfig.json
├── .github/workflows/build.yml
├── PRD.md
└── TECH.md
```

---

## 11. 開發里程碑

| 里程碑 | 內容 | 預估工時 |
|--------|------|---------|
| M1: 骨架 | Tauri v2 初始化、SpecLens 品牌設定 | 3 天 |
| M2: 環境管理 | 安裝/更新檢查、安裝引導 UI | 4 天 |
| M3: 步驟引擎 | 步驟列表、狀態顯示、進度條 | 5 天 |
| M4: Phase Tabs + Overview | 三層子頁結構、OverviewPanel（步驟詳情）| 4 天 |
| M5: Terminal Bridge | PTY + 時間戳緩衝 + 通用高亮規則 | 5 天 |
| M6: 輸出關聯 | `get_output_slice`、OverviewPanel 輸出片段、Task 30秒近似 | 3 天 |
| M7: Documents | PhaseScanner + DocHashStore + DocumentPreview | 4 天 |
| M8: Task File Bar | TaskFilePicker（首次強制）+ TaskFileBar（永遠顯示）+ 多格式解析 | 4 天 |
| M9: Agent Detector | 12 Agent 特徵偵測、AgentBadge | 2 天 |
| M10: FS Watcher | 步驟狀態、文件 hash、任務檔變更全面監控 | 2 天 |
| M11: 持久化 | workspace.json（docHashes + taskFileBar + agentOverride）| 2 天 |
| M12: CI/CD + 打包 | GitHub Actions 三平台建置 | 3 天 |
| M13: 測試 + 修 Bug | 單元 / E2E（含 Picker 首次邏輯）| 4 天 |
| **Total** | | **~49 天** |

---

*文件版本: 1.3.0 | 最後更新: 2026-04-10*  
*變更: 重命名 SpecLens、九項衝突決策全部落地（①Overview子頁 ②Picker首次強制 ③Bar永遠顯示 ④時間戳輸出關聯 ⑤12個Agent ⑥刪除舊Command ⑦Hash判斷DocStatus ⑧通用高亮規則 ⑨雙層安全模型）*
