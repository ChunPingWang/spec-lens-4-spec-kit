# PRD.md — SpecLens
## 產品需求文件 (Product Requirements Document)

**版本**: v1.3.0  
**日期**: 2026-04-10  
**狀態**: Draft  
**產品名稱**: SpecLens  
**目標**: 為 AI 開發者提供 GitHub Spec-Kit 的跨平台視覺化管理工具

---

## 1. 產品概述

### 1.1 背景與動機

GitHub Spec-Kit 是 AI 輔助開發流程的規範工具，透過定義 Spec 步驟指導 AI Agent 依序完成開發任務。然而目前 Spec-Kit 僅提供 CLI 操作，對於需要同時監控多個步驟、追蹤進度、觀察 AI 輸出的開發者而言，缺乏可視化介面是最大的痛點。

**SpecLens** 旨在解決這個問題：以桌面應用程式的形式，提供完整的 Spec-Kit 管理 UI，讓開發者在 AI 開發過程中獲得清晰、即時的視覺回饋——如同光譜儀將白光分解為所有可見色彩，SpecLens 將專案的每個開發階段完整呈現。

### 1.2 目標用戶

| 用戶類型 | 使用情境 |
|---------|---------|
| AI 應用開發者 | 使用 Spec-Kit 驅動 AI Agent 開發軟體 |
| 技術架構師 | 審查 Spec 定義與進度 |
| 全端工程師 | 需要跨平台（Mac / Windows / Linux）操作 |
| DevOps 工程師 | 整合 Spec 流程至 CI/CD 管線 |

### 1.3 核心價值主張

- **可視化進度**：一目瞭然看到 Spec-Kit 全部步驟與當前執行位置
- **階段產出物管理**：每個 Spec-Kit 階段對應獨立 Tab，Overview / Documents / Tasks 三層結構清晰瀏覽
- **Tasks 任務追蹤**：Tasks 階段以互動式清單顯示每項任務的完成狀態，支援自選任務檔路徑與多檔 Bar 切換
- **即時終端橋接**：AI Agent 的 Terminal 輸出直接串流至 UI，步驟與任務自動關聯對應輸出片段
- **全 AI Agent 相容**：支援 Spec-Kit 所有 12 個官方 AI Agent，自動偵測當前專案使用的 Agent 類型
- **自動化環境管理**：啟動時自動偵測並處理 Spec-Kit 的安裝與更新
- **跨平台一致體驗**：Mac / Windows / Linux 統一 UX

---

## 2. 功能需求

### 2.1 專案管理 (Project Management)

#### FR-001: 開啟專案路徑
- 使用者可透過 **檔案選擇器** 或 **拖曳資料夾** 開啟一個本機專案目錄
- 系統讀取該目錄下的 Spec-Kit 設定檔（如 `.speckit/config.yaml` 或 `speckit.json`）
- 支援 **最近開啟的專案** 清單（最多 10 筆），可一鍵重開
- 每個專案以獨立的 **工作區 (Workspace)** 概念管理，記憶上次瀏覽的步驟與 Tab 位置

#### FR-002: 專案狀態顯示
- 顯示當前專案名稱、路徑、最後修改時間
- 顯示 Spec-Kit 版本資訊
- 顯示偵測到的 AI Agent 圖示與名稱

---

### 2.2 Spec-Kit 環境管理 (Environment Management)

#### FR-010: 啟動時環境檢查
每次開啟專案時，系統自動執行環境檢查流程：

```
啟動順序：
1. 檢查 Spec-Kit CLI 是否已安裝（PATH / 常見位置）
2. 若未安裝 → 顯示安裝引導 Dialog，支援一鍵安裝
3. 若已安裝 → 檢查當前版本 vs 最新版本（呼叫 GitHub API）
4. 若有更新 → 顯示更新通知 Banner，提供「立即更新」/ 「略過」選項
5. 環境就緒 → Agent 偵測 → 進入主畫面
```

#### FR-011: 安裝引導
- 顯示 Spec-Kit 安裝指令（依作業系統自動切換）
- 提供在應用內執行安裝的選項，並顯示安裝進度
- 安裝完成後自動回到環境檢查流程

#### FR-012: 版本管理
- 顯示目前安裝版本與最新版本
- 支援 Changelog 預覽（從 GitHub Release Notes 擷取）
- 「更新」操作在背景執行，顯示進度條

---

### 2.3 Spec 步驟總覽 (Steps Overview)

#### FR-020: 全步驟列表
解析目前專案的 Spec-Kit 設定，列出**所有定義的步驟**：
- 步驟編號、名稱 / ID、描述（簡短）
- 步驟類型（plan / implement / review / test）
- 依賴關係（前置步驟）
- 預估執行時間（若設定中有定義）

#### FR-021: 步驟狀態視覺化

| 狀態 | 意義 | 視覺呈現 |
|-----|------|---------|
| `pending` | 尚未執行 | 灰色圓圈 |
| `running` | 執行中 | 藍色動態脈衝圈 |
| `completed` | 成功完成 | 綠色勾選 |
| `failed` | 執行失敗 | 紅色叉號 |
| `skipped` | 已略過 | 黃色虛線圓圈 |
| `blocked` | 等待前置步驟 | 橘色鎖定圖示 |

#### FR-022: 步驟進度條
- 主畫面頂部顯示整體進度條（已完成步驟 / 總步驟數）
- 顯示百分比與「X / Y 步驟完成」文字

---

### 2.4 Phase Tab 與三層子頁結構

#### FR-030: Phase Tab 三層子頁設計

每個 Phase Tab 內部統一採用三層子頁結構：

```
Phase Tab（如 tasks）
  └── [ Overview ] [ Documents ] [ Tasks* ]
                                  *僅 tasks 階段出現
```

**Overview 子頁（步驟詳情）**：
- 步驟名稱、當前狀態、執行時長
- 完整描述、輸入參數（Input Spec）、預期輸出（Expected Output）
- 啟動時間與執行時長
- 歷史模式 toggle：回顧已完成步驟的執行摘要
- **Terminal 輸出關聯區**（可展開摺疊）：
  - 顯示該步驟 `running → completed/failed` 時間區間的 PTY 輸出片段
  - 「在 Terminal 面板中查看完整輸出」捷徑連結

**Documents 子頁**：
- 顯示該階段所有產出文件，含狀態徽章與內嵌預覽（見 FR-032）

**Tasks 子頁**（僅 tasks 階段）：
- Task File Bar + 清單 + 進度條（見 FR-051～054）

#### FR-031: 每個階段對應獨立 Phase Tab

```
[ spec ] [ tasks ] [ implement ] [ review ] [ test ]
   ↑ 當前作用中的 Tab 與左側步驟列表高亮同步
```

- 點擊任一 Tab 可手動切換；左側步驟切換時右側 Tab 自動跟隨
- Tab 上顯示階段狀態小圓點（顏色對應 `StepStatus`）
- 尚未執行的階段 Tab 以灰階顯示，可設定是否鎖定

#### FR-032: 階段產出文件面板（Documents 子頁）

掃描對應階段輸出路徑，每筆文件顯示：
- 檔案名稱與副檔名圖示、最後修改時間、檔案大小
- 文件狀態徽章：
  - `generated`：FS Watcher 首次 Create 事件後，SHA-256 hash 未變動
  - `modified`：SHA-256 hash 與首次記錄不同
  - `missing`：預期存在但未找到
  - ⓘ 冷啟動無歷史記錄時，顯示「無法確認原始狀態」提示

點擊文件可在內嵌預覽區開啟（Markdown 渲染、語法高亮、JSON/YAML 美化）。

---

### 2.5 AI Agent 終端輸出橋接 (Terminal Bridge)

#### FR-040: 終端輸出串流
- UI 底部 Terminal 面板即時顯示 PTY 輸出（高度可拖曳）
- 支援 ANSI 顏色代碼渲染
- 每行輸出附帶 timestamp_ms，用於步驟輸出關聯

#### FR-041: 輸出過濾與搜尋
- 關鍵字過濾（包含/排除）、Log Level 過濾（INFO / WARN / ERROR）
- 搜尋框支援 RegEx

#### FR-042: 輸出操作
- 一鍵複製至剪貼簿；匯出為 `.log` / `.txt`（檔頭附  ASCII 標注）
- 清空畫面（不刪除歷史緩衝）；自動捲動（可鎖定 / 解鎖）

#### FR-043: 終端互動（進階，可選功能）
- 設定開關：支援在 UI 內輸入指令，轉發至 PTY 執行
- 常用指令快速按鈕：`speckit run`、`speckit status`、`speckit reset`

#### FR-044: 通用視覺高亮規則

對**所有 AI Agent** 套用以下通用高亮（不依賴特定 Agent 格式，不做語意解析）：

| 模式類型 | 觸發條件 | 視覺效果 |
|---------|---------|---------|
| 成功 | 行首 `✓` `✅` `✔` `[done]` `[ok]` | 綠色淡底色 |
| 錯誤 | 行首 `✗` `❌` `Error` `Failed` `FAILED` | 紅色淡底色 |
| 進行中 | 行首 `→` `▶` `Running` `[...]` | 藍色文字 |
| 警告 | 行首 `⚠` `Warning` `WARN` | 黃色文字 |
| JSON 區塊 | 整行為合法 JSON 物件 | 語法高亮 |
| 檔案路徑 | `./` 或 `/` 開頭 | 底線 + 可點擊開啟預覽 |

規則以 regex 設定檔定義，使用者可自訂新增或修改。

---

### 2.6 Tasks 階段任務清單 (Tasks Phase Checklist)

#### FR-050: Tasks 任務檔來源設定（可選路徑與檔案）

**核心原則：不強制約定任務檔位置，由使用者完全掌控來源。**

首次切換至 `tasks` Tab，一律先顯示 **Task File Picker**，絕不自動載入：

```
┌─ 選擇任務檔來源 ────────────────────────────────────────────┐
│  掃描結果（自動偵測，需手動勾選）：                              │
│  ☐ .agent/features/001-auth/tasks.md    （建議）             │
│  ☐ .agent/features/002-api/tasks.md                         │
│  ☐ docs/tasks-v2.md                                          │
│  或手動指定：  [📁 選擇路徑]  [📄 選擇檔案]                    │
│  支援格式：  .md  .json  .yaml  .txt                         │
│                              [取消]  [套用選取項目]            │
└──────────────────────────────────────────────────────────────┘
```

- 掃描結果僅供選擇，不自動勾選
- 選擇結果持久化至專案設定（下次開啟自動載入已選清單）
- 右上角常駐「⚙ 變更來源」按鈕

#### FR-051: Task File Bar（多檔橫向 Bar）

**無論任務檔數量，Bar 永遠顯示。**

- 1 個 Tab：隱藏捲動箭頭與關閉按鈕，行為與多檔一致
- 每個 Tab 顯示：檔名（含上層目錄）、完成狀態徽章、關閉按鈕（×）
- 超出可視範圍時顯示 `◀ ▶` 捲動箭頭
- 「+ 新增檔案」追加更多任務檔，各 Tab 篩選狀態獨立維護

#### FR-052: Tasks 進度摘要列

```
（多個任務檔）
全部合計   ████████████████░░░░  19 / 41 完成  (46%)
當前檔案   ██████████░░░░░░░░░    7 / 15 完成  (47%)

（單一任務檔）
Tasks 進度 ████████████░░░░░░░   7 / 15 完成  (47%)
```

進度條顏色：0–49% 橘黃、50–79% 藍、80–100% 綠

#### FR-053: Tasks 篩選與搜尋

```
[ 全部 (15) ]  [ ✅ 已完成 (7) ]  [ ☐ 未完成 (8) ]    🔍 搜尋...
```

作用範圍為當前 Tab；未完成 Task 排列在上方。

#### FR-054: Task 詳情展開

展開詳情包含：完整描述、相關文件（可點擊跳轉）、**Terminal 近似輸出片段**（完成前 30 秒，標注「≈ 完成前 30 秒輸出」）、失敗時的錯誤摘要。

---

### 2.7 AI Agent 支援矩陣 (AI Agent Compatibility)

#### FR-060: 支援所有 Spec-Kit 官方 AI Agent（12 個）

| Agent | 開發商 | `--ai` 參數 | 偵測特徵 |
|-------|--------|------------|---------|
| **Claude Code** | Anthropic | `claude` | `.claude/` / `CLAUDE.md` |
| **GitHub Copilot** | Microsoft / GitHub | `copilot` | `.github/copilot-instructions.md` |
| **Gemini CLI** | Google | `gemini` | `GEMINI.md` / `.gemini/` |
| **Cursor** | Anysphere | `cursor` | `.cursor/` / `.cursorrules` |
| **Windsurf** | Codeium | `windsurf` | `.windsurfrules` |
| **Amazon Q Developer** | AWS | `q` | `.amazonq/` |
| **Codex CLI** | OpenAI | `codex` | `AGENTS.md`（codex 格式）|
| **Qwen Code** | Alibaba | `qwen` | `.qwen/` |
| **opencode** | SST | `opencode` | `opencode.json` |
| **Kilo Code** | Kilo | `kilocode` | `.kilocode/` |
| **Auggie CLI** | Augment Code | `auggie` | `.augment/` |
| **Roo Code** | Roo Cline | `roo` | `.roo/` / `.roosettings` |

> **Agent 無關設計**：未列出的 Agent 仍可透過通用 PTY 橋接使用全部功能。

#### FR-061: Agent 偵測流程

```
開啟專案
  ├─ 讀取 specify.yaml --ai 記錄 → 顯示對應 Agent 圖示（最高優先）
  ├─ 掃描特徵檔案             → 顯示 Agent 圖示 + 「自動偵測」標籤
  └─ 未偵測                  → 顯示「通用模式」圖示，全部功能可用
```

---

### 2.8 通知與警告 (Notifications)

#### FR-070: 系統通知
- Spec 步驟完成 / 失敗時，發送 OS 原生通知（可設定關閉）
- 環境檢查結果以 Toast 通知呈現

---

## 3. 非功能需求

### 3.1 效能
- 應用啟動時間 < 2 秒（冷啟動）
- 終端輸出渲染延遲 < 100ms
- 步驟清單滾動幀率 ≥ 60fps

### 3.2 可用性
- 鍵盤快捷鍵：開啟專案 `Cmd/Ctrl+O`、清除輸出 `Cmd/Ctrl+K`
- 淺色 / 深色主題自動切換（跟隨 OS）
- Terminal 面板字體大小可調整

### 3.3 跨平台

| 平台 | 支援版本 |
|-----|---------|
| macOS | 12 Monterey 以上（Apple Silicon & Intel） |
| Windows | Windows 10 / 11（x64） |
| Linux | Ubuntu 22.04+、Debian 12+、Fedora 38+（x64） |

### 3.4 安全性（雙層模型）

| 操作類型 | 風險等級 | 限制範圍 |
|---------|---------|---------|
| PTY 命令執行 | 🔴 高 | `cwd` 強制鎖定在專案目錄 |
| 檔案寫入（匯出等）| 🟡 中 | 限制在專案目錄內 |
| 檔案唯讀（任務檔、預覽）| 🟢 低 | 允許任意路徑（OS 選擇器授權）|

所有設定儲存於 OS 原生 Keychain / Secret Store；不收集使用者資料，完全本機運算。

---

## 4. 使用者旅程 (User Journey)

```
[啟動應用] → SpecLens Logo +
    ▼
[歡迎畫面 / 最近專案列表]
    ▼
[環境檢查]
  ├─ 未安裝 → [安裝引導] → [重新檢查]
  ├─ 有更新 → [更新 Banner]
  └─ 就緒
    ▼
[Agent 偵測] → 頂部顯示 Agent 圖示
    ▼
[主畫面]
┌──────────────────────────────────────────────────────────────────┐
│ [🤖 Claude Code]  SpecLens  my-project  ████████░░ 6/8  ││
├──────────────┬───────────────────────────────────────────────────┤
│              │ [ spec✅ ][ tasks🔄 ][ implement ][ review ]       │
│  步驟列表    │ ─────────────────────────────────────────────     │
│  (可捲動)    │ 子頁：[ Overview ] [ Documents ] [ Tasks ]         │
│              │                                                     │
│  ● spec  ✅  │ （Tasks 子頁）                                      │
│  ● tasks 🔄  │ ⚙ 來源  [+ 新增]                                   │
│  ● impl  ⬜  │ ◀ [ 001-auth ✅12/12 ][ 002-api 🔄7/15 ] ▶        │
│  ● review ⬜ │ 全部合計 ███████░░  19/41                          │
│              │ 當前檔案 ████░░░░   7/15                           │
│              │ [ 全部 ][ ✅ ][ ☐ ]  🔍   ✅#001  ☐#002            │
├──────────────┴───────────────────────────────────────────────────┤
│ Terminal  [Filter][Search][Copy][Export][Clear]  🔒 捲動          │
└──────────────────────────────────────────────────────────────────┘
                                       
```

---

## 5. 成功指標 (Success Metrics)

| 指標 | 目標值 |
|-----|-------|
| 應用啟動成功率 | ≥ 99.5% |
| 終端輸出串流延遲 | ≤ 100ms P95 |
| 步驟→輸出關聯準確率 | ≥ 90% |
| 跨平台 Bug 比例 | < 5% of total issues |
| 使用者滿意度（NPS） | ≥ 40 |

---

## 6. 範疇外 (Out of Scope - v1.0)

- 雲端同步 / 遠端專案
- 多使用者協作
- Spec-Kit 設定檔的視覺化編輯器
- AI 模型直接整合（僅監控，不控制 AI Agent）
- Agent 特定輸出語意解析（已改為通用高亮規則 FR-044）
- 行動裝置版本

---

*文件版本: 1.3.0 | 最後更新: 2026-04-10*  
*變更: 重命名為 SpecLens、九項衝突決策全部落地（①②③④⑤⑥⑦⑧⑨）*
