# CSV Compare Desktop Tool — SPEC.md

## 1. 專案目標

建立一套以 **Tauri 2 + Rust** 開發的 Windows 免安裝桌面程式，用於比對兩個 CSV / Delimited Text 檔案的內容是否一致，並清楚列出所有差異，最後可匯出 Excel `.xlsx` 比較結果。

產品定位：

> 一套完全離線、免安裝、支援繁體中文編碼、多種分隔符號、Quoted CSV、排除欄位、單一 Key 與複合 Key 比對的資料核對工具。

主要使用情境：

- 金融報表核對
- 內控 / 稽核資料核對
- 系統轉檔驗證
- ETL 驗證
- 批次結果驗證
- 資料庫匯出資料驗證
- 兩版資料差異比對

---

## 2. 核心需求摘要

系統必須支援：

1. 指定 File A 與 File B 的檔案路徑。
2. 支援 `.csv` 與 `.txt` 等 Delimited Text File。
3. 自動偵測並手動指定文字編碼。
4. 支援主流繁體中文編碼。
5. 自動偵測並手動指定分隔符號。
6. File A 與 File B 可使用不同分隔符號。
7. 正確處理雙引號包覆欄位。
8. 正確處理欄位內的分隔符號、雙引號與換行。
9. 自動解析 Header。
10. 使用者可指定排除比較欄位。
11. 使用者可指定一個或多個關鍵欄位作為 Key。
12. 支援 Composite Key。
13. 使用 Key 比對時忽略資料列順序。
14. 支援 Row-by-Row 比對模式。
15. 偵測 A Only、B Only、Value Changed、Duplicate Key。
16. 顯示差異明細與摘要。
17. 匯出 Excel `.xlsx` 比較結果。
18. 完全於本機處理資料。
19. 不需要 Server、Database、API、Internet。
20. 支援至少 100 MB 級別檔案。
21. 比較過程不得造成 UI Freeze。
22. 支援取消比較。

---

# 3. 技術架構

## 3.1 技術棧

使用：

- Tauri 2.x
- Rust
- TypeScript
- React（建議，但非強制）

建議：

```text
Tauri 2
+ React
+ TypeScript
+ Rust
```

核心資料處理必須由 Rust Backend 負責。

Frontend 不應負責：

- CSV Parsing
- Encoding Conversion
- Delimiter Detection
- Data Comparison
- Excel Generation

---

## 3.2 Architecture

```text
Tauri Desktop App
│
├── Frontend
│   ├── File Selection
│   ├── Encoding Settings
│   ├── Delimiter Settings
│   ├── Key Selection
│   ├── Excluded Column Selection
│   ├── Compare Settings
│   ├── Progress UI
│   ├── Result Summary
│   └── Difference Table
│
└── Rust Backend
    ├── File Reader
    ├── Encoding Detector
    ├── Encoding Converter
    ├── Delimiter Detector
    ├── CSV / Delimited Parser
    ├── Header Analyzer
    ├── Key Builder
    ├── Data Comparator
    ├── Duplicate Key Detector
    ├── Difference Generator
    └── Excel Exporter
```

---

# 4. 執行環境

主要支援：

```text
Windows 10 64-bit
Windows 11 64-bit
```

程式需提供 Portable 版本。

預期使用方式：

```text
CSVCompare.exe
```

使用者：

```text
下載 / 複製程式
↓
直接執行 CSVCompare.exe
↓
開始使用
```

不得要求：

- Administrator 權限
- MSI 安裝
- Python
- Java
- Node.js
- .NET Runtime 額外安裝
- Windows Service
- Database
- Server
- API Key
- Internet Connection

---

# 5. 檔案支援

支援：

```text
.csv
.txt
```

只要內容符合 Delimited Text Format 即可。

File Picker 預設支援：

```text
CSV Files (*.csv)
Text Files (*.txt)
All Files (*.*)
```

使用者可以：

1. 使用 Windows File Picker。
2. 手動輸入完整檔案路徑。

---

# 6. File A / File B

主畫面提供：

```text
File A
[C:\Data\A.csv                  ] [Browse]

Encoding：
[Auto - CP950 ▼]

Delimiter：
[Auto - Comma (,) ▼]
```

以及：

```text
File B
[C:\Data\B.txt                  ] [Browse]

Encoding：
[Auto - UTF-8 ▼]

Delimiter：
[Auto - Pipe (|) ▼]
```

File A 與 File B：

- 可以使用不同 Encoding。
- 可以使用不同 Delimiter。
- 可以使用不同 Column Order。
- 使用 Key-based Compare 時可以使用不同 Row Order。

---

# 7. Encoding 支援

至少必須支援：

```text
UTF-8
UTF-8 BOM
Big5
CP950 / Windows-950
UTF-16 LE
UTF-16 BE
```

其中：

```text
CP950
```

為台灣 Windows / Excel 常見繁體中文編碼，屬於必要功能。

---

# 8. Encoding Detection

自動偵測流程建議：

```text
BOM Detection
      ↓
UTF-8 Validation
      ↓
Encoding Detection
      ↓
Big5 / CP950 Fallback
```

建議 Rust Library：

```text
encoding_rs
chardetng
```

UI 顯示：

```text
Encoding：CP950
```

若無法可靠判斷：

```text
無法可靠判斷文字編碼，
請手動指定 Encoding。
```

---

# 9. 手動 Encoding

使用者可覆寫：

```text
Auto
UTF-8
Big5
CP950
UTF-16 LE
UTF-16 BE
```

---

# 10. Delimiter 支援

至少支援：

```text
Comma       ,
Semicolon   ;
Tab         \t
Pipe        |
Colon       :
Custom      使用者指定單一字元
```

Custom 範例：

```text
^
#
~
```

---

# 11. Delimiter Detection

載入檔案時，系統應自動偵測：

```text
,
;
TAB
|
:
```

不得只看第一行某個字元是否出現。

建議分析前：

```text
20～100 個 Logical Records
```

偵測條件：

- 每筆資料欄位數是否穩定。
- Candidate delimiter 出現頻率。
- Quote 規則是否成立。
- Escaped Quote 是否成立。
- 是否能產生一致欄位結構。

偵測結果：

```text
Delimiter：Comma (,)
```

或：

```text
Delimiter：Pipe (|)
```

---

# 12. 手動 Delimiter

UI：

```text
分隔符號

○ Auto
○ Comma (,)
○ Semicolon (;)
○ Tab
○ Pipe (|)
○ Colon (:)
○ Custom
```

選擇 Custom 時：

```text
自訂分隔符：[ ^ ]
```

V1 自訂分隔符以：

```text
單一 Unicode 字元
```

為原則。

---

# 13. Delimited File 格式支援

以下格式皆需支援。

## 13.1 Comma

```csv
編號,ID
123,ABC
456,DEF
```

---

## 13.2 全欄位使用雙引號

必須支援：

```csv
"編號","ID"
"123","ABC"
"456","DEF"
```

解析後：

```text
編號 | ID
123  | ABC
456  | DEF
```

雙引號不得成為實際資料內容。

---

## 13.3 部分欄位使用雙引號

```csv
編號,"姓名","地址"
123,"王小明","台北市,中正區"
```

解析：

```text
編號 = 123
姓名 = 王小明
地址 = 台北市,中正區
```

---

## 13.4 Semicolon

```text
"編號";"ID"
"123";"ABC"
```

---

## 13.5 Pipe

```text
"編號"|"ID"
"123"|"ABC"
```

---

## 13.6 Tab

實際資料：

```text
編號<TAB>ID
123<TAB>ABC
```

---

## 13.7 Colon

```text
"編號":"ID"
"123":"ABC"
```

---

# 14. Quoted Field 規則

Parser 必須正確支援 RFC 4180 常見規則。

---

## 14.1 欄位包含 Delimiter

```csv
"編號","地址"
"001","台北市,中正區"
```

結果：

```text
地址 = 台北市,中正區
```

不可拆成兩欄。

---

## 14.2 欄位包含雙引號

```csv
"編號","備註"
"001","客戶說""今天會付款"""
```

解析：

```text
客戶說"今天會付款"
```

---

## 14.3 欄位包含換行

```csv
"編號","備註"
"001","第一行
第二行"
```

必須視為同一筆資料、同一欄位。

因此系統中的 Row Number 應區分：

- Logical Record Number
- Physical Line Number（僅錯誤診斷需要時使用）

一般比對結果以 Logical Record Number 為主。

---

# 15. CSV Parser

禁止：

```text
split(",")
split("|")
split(";")
```

等手工字串切割方式。

必須使用正式 Parser。

建議 Rust：

```text
csv crate
```

Parser 必須依據：

```text
Delimiter
Quote
Escape
Encoding
```

進行正確解析。

---

# 16. Header

預設：

```text
第一筆 Logical Record 為 Header
```

例如：

```csv
"編號","ID","姓名","金額"
"123","ABC","王小明","1000"
```

Header：

```text
編號
ID
姓名
金額
```

雙引號需由 Parser 移除。

---

# 17. Header Mapping

欄位比對不得依：

```text
Column Position
```

而應依：

```text
Header Name
```

例如 File A：

```text
"ID","姓名","金額"
```

File B：

```text
"姓名","金額","ID"
```

應建立：

```text
ID   ↔ ID
姓名 ↔ 姓名
金額 ↔ 金額
```

Column Order 不同不應直接判定資料不同。

---

# 18. Header 差異

系統必須分析：

```text
Common Columns
A Only Columns
B Only Columns
```

例如：

File A：

```text
ID
姓名
金額
```

File B：

```text
ID
姓名
日期
```

結果：

```text
Common：
ID
姓名

A Only：
金額

B Only：
日期
```

---

# 19. Duplicate Header

若同一檔案出現重複 Header，例如：

```csv
"ID","姓名","ID"
```

不得自動猜測。

應阻止正式比較並顯示：

```text
發現重複欄位名稱：

ID

請先修正來源檔案。
```

原因：

```text
Key Mapping 與 Column Mapping 會產生歧義。
```

---

# 20. 比對模式

系統需支援兩種模式：

```text
1. Key-based Compare
2. Row-by-Row Compare
```

建議預設：

```text
Key-based Compare
```

前提是使用者已指定 Key。

UI：

```text
比對模式

● 使用 Key 比對
○ 依資料列順序比對
```

---

# 21. Key 欄位

使用者可以指定：

```text
一個或多個欄位
```

作為資料的唯一 Key。

例如：

```text
☑ 客戶編號
☑ 交易日期
☑ 交易序號
☐ 姓名
☐ 金額
```

Composite Key：

```text
客戶編號 + 交易日期 + 交易序號
```

---

# 22. 單一 Key

File A：

```csv
"客戶編號","姓名","金額"
"001","王小明","1000"
"002","陳小華","2000"
```

File B：

```csv
"客戶編號","姓名","金額"
"002","陳小華","2000"
"001","王小明","1000"
```

Key：

```text
客戶編號
```

配對：

```text
001 ↔ 001
002 ↔ 002
```

結果：

```text
✅ 資料完全相同
```

Row Order 不影響結果。

---

# 23. Composite Key

Key：

```text
客戶編號
交易日期
交易序號
```

資料：

```csv
"客戶編號","交易日期","交易序號","金額"
"001","2026/09/18","01","1000"
"001","2026/09/18","02","2000"
```

唯一 Key：

```text
001 / 2026/09/18 / 01
001 / 2026/09/18 / 02
```

---

# 24. Composite Key 內部表示

禁止直接：

```text
key = col1 + col2 + col3
```

避免：

```text
A + BC
```

與：

```text
AB + C
```

都變成：

```text
ABC
```

Rust 建議：

```rust
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct CompositeKey(Vec<String>);
```

例如：

```rust
CompositeKey(vec![
    "001".to_string(),
    "2026/09/18".to_string(),
    "01".to_string(),
])
```

---

# 25. Key 可選欄位

Key 只能從：

```text
File A 與 File B 都存在的共同 Header
```

中選擇。

例如：

A：

```text
ID
姓名
金額
```

B：

```text
ID
姓名
日期
```

可作 Key：

```text
ID
姓名
```

不可：

```text
金額
日期
```

---

# 26. Key 與 Excluded Columns

Key 欄位不得同時設定為：

```text
Excluded Column
```

若某欄位已選為 Key：

```text
Key：
客戶編號
```

則排除比較區中的：

```text
客戶編號
```

應 Disable。

---

# 27. Key 順序

Composite Key 的內部欄位順序應固定。

建議採：

```text
File A Header 原始順序
```

而不是使用者 Checkbox 點擊順序。

例如 Header：

```text
交易日期
客戶編號
交易序號
```

即使使用者勾選順序不同，實際 Composite Key 仍為：

```text
交易日期 + 客戶編號 + 交易序號
```

---

# 28. Key Strict Rules

Key 預設使用 Strict Mode。

例如：

```text
001 != 1
ABC != abc
ABC != "ABC "
```

所有 Key Value 都以：

```text
String
```

處理。

避免：

- 前導零消失
- 帳號被轉數字
- 編碼代碼失真
- 科學記號轉換

---

# 29. Empty / Incomplete Key

單一 Key：

```csv
"ID","姓名"
"","王小明"
```

需顯示：

```text
⚠ Empty Key
```

Composite Key 任一欄位為空：

```text
客戶編號 = 001
交易日期 = ""
交易序號 = 01
```

需標記：

```text
INCOMPLETE_KEY
```

空值不應自動轉成 NULL、0、N/A。

---

# 30. Duplicate Key

Duplicate Key Detection 為必要功能。

例如 Key：

```text
ID
```

File A：

```csv
"ID","姓名"
"001","王小明"
"001","陳小華"
```

結果：

```text
⚠ Duplicate Key

Source：File A
Key：001
Count：2
```

不得：

```text
用後一筆覆蓋前一筆
```

---

# 31. Duplicate Composite Key

Key：

```text
客戶編號
交易日期
交易序號
```

以下為 Duplicate：

```text
001 / 2026/09/18 / 01
001 / 2026/09/18 / 01
```

以下不是：

```text
001 / 2026/09/18 / 01
001 / 2026/09/18 / 02
```

---

# 32. Duplicate Key 處理原則

若任一檔案出現 Duplicate Key：

預設：

```text
繼續完成資料掃描
但 Key-based Compare 結果標記為 HAS_DUPLICATE_KEYS
```

對該 Duplicate Key：

```text
不得任意選其中一筆配對
```

應列入：

```text
Duplicate_Keys
```

報告，並提示使用者此 Key 無法唯一對應。

---

# 33. Key-based Compare 流程

```text
讀取 File A
      ↓
解析 Encoding
      ↓
解析 Delimiter
      ↓
解析 Header
      ↓
建立 Composite Key
      ↓
建立 File A Key Index
      ↓
檢查 Duplicate Key
      ↓
Streaming File B
      ↓
建立 File B Composite Key
      ↓
依 Key 尋找 File A Record
      ↓
比較所有非排除欄位
      ↓
產生 Difference
      ↓
找出 A Only
      ↓
完成 Summary
```

---

# 34. Row-by-Row Compare

若選擇：

```text
依資料列順序比對
```

則：

```text
A Row 1 ↔ B Row 1
A Row 2 ↔ B Row 2
A Row 3 ↔ B Row 3
```

此模式不需要 Key。

但：

```text
Column 仍依 Header Name Mapping
```

而不是單純依 Column Index。

---

# 35. 排除比較欄位

使用者可指定不納入 Value Comparison 的欄位。

例如：

```text
☐ 客戶編號
☐ 姓名
☐ 金額
☑ 更新時間
☑ 批次日期
```

則：

```text
更新時間
批次日期
```

不影響：

```text
Record Same / Different
```

判定。

---

# 36. Excluded Column UI

提供：

```text
搜尋欄位
全選
全部取消
反向選取
```

Key 欄位：

```text
Disabled
```

不可排除。

---

# 37. Value Comparison Rules

預設：

```text
Strict String Compare
```

例如：

```text
ABC != abc
ABC != ABC<space>
00123 != 123
"" != " "
"" != "NULL"
"" != "N/A"
```

所有來源值原則上保持 String。

---

# 38. Optional Compare Settings

可於 V1 提供：

```text
☐ 忽略前後空白
☐ 忽略大小寫
```

預設：

```text
Off
```

如啟用：

```text
Normalize 僅影響比較
不得修改原始顯示值
不得修改 Excel 原始值
```

---

# 39. Difference Types

至少支援：

```text
VALUE_CHANGED
A_ONLY
B_ONLY
COLUMN_A_ONLY
COLUMN_B_ONLY
DUPLICATE_KEY_A
DUPLICATE_KEY_B
EMPTY_KEY_A
EMPTY_KEY_B
INCOMPLETE_KEY_A
INCOMPLETE_KEY_B
```

---

# 40. VALUE_CHANGED

例如：

File A：

```csv
"ID","姓名","金額"
"001","王小明","1000"
```

File B：

```csv
"ID","姓名","金額"
"001","王小明","1200"
```

Key：

```text
ID
```

結果：

```text
Key：001
Column：金額
File A：1000
File B：1200
Type：VALUE_CHANGED
```

---

# 41. A_ONLY

File A 有：

```text
ID = 001
```

File B 找不到：

```text
ID = 001
```

結果：

```text
Key：001
Type：A_ONLY
```

---

# 42. B_ONLY

File B 有：

```text
ID = 005
```

File A 找不到：

```text
ID = 005
```

結果：

```text
Key：005
Type：B_ONLY
```

---

# 43. Column Difference

例如：

File A：

```text
ID
姓名
金額
```

File B：

```text
ID
姓名
日期
```

結果：

```text
COLUMN_A_ONLY：金額
COLUMN_B_ONLY：日期
```

共同欄位仍可繼續比較。

---

# 44. 比較結果 Summary

範例：

```text
比較完成

File A
A.csv

Encoding
CP950

Delimiter
Comma (,)

File B
B.txt

Encoding
UTF-8

Delimiter
Pipe (|)

Compare Mode
KEY_BASED

Key Columns
客戶編號
交易日期
交易序號

Rows A：10,000
Rows B：10,020

Matched Records：9,950
Same Records：9,900
Different Records：50

A Only：50
B Only：70

Different Cells：72

Duplicate Keys A：0
Duplicate Keys B：0

結果：
❌ 資料內容不同
```

完全一致：

```text
✅ 兩個檔案內容完全相同
```

---

# 45. Difference Table UI

Key-based Compare 時：

| Key | Column | File A | File B | Type |
|---|---|---|---|---|
| ID=001 | 金額 | 1000 | 1200 | VALUE_CHANGED |
| ID=005 | - | - | Record | B_ONLY |

Composite Key 顯示：

```text
客戶編號=001
交易日期=2026/09/18
交易序號=02
```

或簡潔顯示：

```text
001 | 2026/09/18 | 02
```

---

# 46. 搜尋與 Filter

Difference UI 提供：

```text
搜尋
```

可搜尋：

- Key
- Row Number
- Column Name
- File A Value
- File B Value
- Difference Type

Filter：

```text
全部
Value Changed
A Only
B Only
Column Difference
Duplicate Key
Key Error
```

---

# 47. Excel 匯出

比較完成後提供：

```text
[匯出 Excel]
```

輸出：

```text
.xlsx
```

禁止：

```text
.xls
```

建議 Rust Library：

```text
rust_xlsxwriter
```

---

# 48. Excel Workbook

預設檔名：

```text
CSV_Compare_Result.xlsx
```

至少包含：

```text
Summary
Differences
Column_Differences
Excluded_Columns
Duplicate_Keys
```

---

# 49. Summary Sheet

至少包含：

| Item | Result |
|---|---|
| File A | C:\Data\A.csv |
| File A Encoding | CP950 |
| File A Delimiter | Comma |
| File B | C:\Data\B.txt |
| File B Encoding | UTF-8 |
| File B Delimiter | Pipe |
| Compare Mode | KEY_BASED |
| Key Columns | 客戶編號 / 交易日期 / 交易序號 |
| Rows A | 10000 |
| Rows B | 10020 |
| Matched Records | 9950 |
| Same Records | 9900 |
| Different Records | 50 |
| A Only | 50 |
| B Only | 70 |
| Different Cells | 72 |
| Duplicate Keys A | 0 |
| Duplicate Keys B | 0 |
| Result | DIFFERENT |
| Compare Time | 2026-09-18 19:30:00 |

---

# 50. Differences Sheet

若 Key：

```text
客戶編號
交易日期
交易序號
```

Excel 應將 Key 分開成欄位：

| 客戶編號 | 交易日期 | 交易序號 | Row A | Row B | Column | File A Value | File B Value | Type |
|---|---|---|---:|---:|---|---|---|---|
| 001 | 2026/09/18 | 02 | 25 | 80 | 金額 | 2000 | 2500 | VALUE_CHANGED |

不得只把 Composite Key 合併成單一文字欄位。

---

# 51. Column_Differences Sheet

| Column | File A | File B | Status |
|---|---|---|---|
| ID | Yes | Yes | SAME |
| 姓名 | Yes | Yes | SAME |
| 金額 | Yes | No | A_ONLY |
| 日期 | No | Yes | B_ONLY |

---

# 52. Excluded_Columns Sheet

例如：

| Column |
|---|
| 更新時間 |
| 批次日期 |
| 匯入時間 |

---

# 53. Duplicate_Keys Sheet

例如：

| Source | 客戶編號 | 交易日期 | 交易序號 | Count | Rows |
|---|---|---|---|---:|---|
| File A | 001 | 2026/09/18 | 01 | 2 | 25, 31 |

若無：

```text
No duplicate keys found.
```

---

# 54. Excel 原始值保護

CSV 原始值原則上以：

```text
Text
```

寫入 Excel。

例如：

```text
001234
```

匯出後仍須：

```text
001234
```

不可變成：

```text
1234
```

同樣避免：

- 日期自動轉型
- 大型帳號轉科學記號
- 長整數精度遺失
- 前導零遺失

---

# 55. Excel 視覺格式

至少：

- Header Bold
- Freeze Header Row
- Auto Filter
- 適當 Column Width
- Difference Type 清楚呈現
- Summary 易讀
- 所有原始資料欄位以文字格式寫入

---

# 56. 大檔案支援

目標：

```text
至少可處理：
File A 100 MB
+
File B 100 MB
```

不得簡單：

```rust
std::fs::read_to_string(...)
```

將兩個完整大檔同時載入 String。

應使用：

```text
BufReader
Streaming Decoder
Streaming CSV Parser
```

---

# 57. Key-based Memory Strategy

基本策略：

```text
File A
↓
Streaming Parse
↓
建立 Key Index
↓
File B
↓
Streaming Parse
↓
逐筆 Key Lookup
↓
Compare
```

可使用：

```rust
HashMap<CompositeKey, Record>
```

但需避免：

```text
File A + File B 全部同時建立完整 Vec<Row>
```

若 File A 較大、File B 較小，可進一步選擇較小檔案作為 Index Source。

---

# 58. Progress

比較期間不得凍結 UI。

Rust Backend 應執行於 Background Task。

顯示：

```text
正在比較...
```

Progress：

```text
████████████░░░░ 67%
```

以及：

```text
已處理 67,000 / 100,000 Records
```

若無法事先取得 Total Records，可顯示：

```text
已處理 67,000 Records
```

---

# 59. Cancel

提供：

```text
[取消比較]
```

取消後：

```text
比較已取消
```

不得：

- Crash
- 留下損壞 Excel
- 留下鎖定中的來源檔案

---

# 60. Error Handling

所有 Error 必須：

```text
可理解
可處理
不可造成 App Crash
```

---

## 60.1 File Not Found

```text
找不到檔案：

C:\Data\A.csv
```

---

## 60.2 File Access Error

```text
無法讀取檔案。

請確認檔案是否存在，以及目前帳號是否具有讀取權限。
```

---

## 60.3 Encoding Error

```text
無法可靠判斷文字編碼。

請手動指定：
UTF-8 / Big5 / CP950 / UTF-16 LE / UTF-16 BE
```

---

## 60.4 Delimiter Error

```text
無法可靠判斷欄位分隔符號。

請手動指定 Delimiter。
```

---

## 60.5 Invalid Record

例如 Header 10 欄，但某筆資料解析為 12 欄：

```text
檔案格式異常。

Record：152
預期欄位：10
實際欄位：12
```

---

## 60.6 Duplicate Header

```text
發現重複欄位名稱：

ID

請修正來源檔案後重新執行。
```

---

## 60.7 Missing Key Column

```text
Key 欄位不存在於兩個檔案：

交易序號
```

---

## 60.8 Export Error

```text
Excel 報告無法儲存。

請確認：
1. 輸出資料夾具有寫入權限
2. 同名 Excel 是否正在開啟
3. 磁碟空間是否足夠
```

---

# 61. Main UI

建議：

```text
┌──────────────────────────────────────────────┐
│ CSV Compare                                  │
├──────────────────────────────────────────────┤
│                                              │
│ File A                                       │
│ [C:\Data\A.csv                   ] [Browse]   │
│ Encoding  [Auto - CP950 ▼]                   │
│ Delimiter [Auto - Comma (,) ▼]               │
│                                              │
│ File B                                       │
│ [C:\Data\B.txt                   ] [Browse]   │
│ Encoding  [Auto - UTF-8 ▼]                   │
│ Delimiter [Auto - Pipe (|) ▼]                │
│                                              │
│ ───────────────────────────────────────────  │
│                                              │
│ 比對模式                                     │
│ ● 使用 Key 比對                              │
│ ○ 依資料列順序比對                          │
│                                              │
│ Key 欄位                                     │
│ [搜尋欄位____________]                       │
│ ☑ 客戶編號                                  │
│ ☑ 交易日期                                  │
│ ☑ 交易序號                                  │
│ ☐ 姓名                                      │
│ ☐ 金額                                      │
│                                              │
│ Composite Key                               │
│ 客戶編號 + 交易日期 + 交易序號              │
│                                              │
│ 排除比較欄位                                 │
│ [搜尋欄位____________]                       │
│ ☐ 姓名                                      │
│ ☐ 金額                                      │
│ ☑ 更新時間                                  │
│ ☑ 批次日期                                  │
│                                              │
│ ☐ 忽略前後空白                              │
│ ☐ 忽略大小寫                                │
│                                              │
│                 [開始比較]                   │
└──────────────────────────────────────────────┘
```

---

# 62. Result UI

```text
┌──────────────────────────────────────────────┐
│ 比較結果                                     │
├──────────────────────────────────────────────┤
│                                              │
│ ❌ 資料內容不同                              │
│                                              │
│ Compare Mode：KEY_BASED                      │
│ Key：客戶編號 + 交易日期 + 交易序號         │
│                                              │
│ Rows A：10,000                               │
│ Rows B：10,020                               │
│ Same：9,900                                  │
│ Different：50                                │
│ A Only：50                                   │
│ B Only：70                                   │
│ Duplicate A：0                               │
│ Duplicate B：0                               │
│                                              │
│ [搜尋________________________]               │
│ [全部 ▼]                                    │
│                                              │
│ Key          Column  A       B       Type     │
│ 001|...|02   金額    2000    2500    CHANGED │
│                                              │
│ [重新比較]                 [匯出 Excel]       │
└──────────────────────────────────────────────┘
```

---

# 63. Rust Module Architecture

建議：

```text
src-tauri/src/
│
├── main.rs
├── lib.rs
│
├── commands/
│   ├── inspect_file.rs
│   ├── compare_files.rs
│   ├── export_excel.rs
│   └── cancel_compare.rs
│
├── parser/
│   ├── encoding.rs
│   ├── delimiter.rs
│   ├── decoder.rs
│   ├── reader.rs
│   └── delimited_parser.rs
│
├── compare/
│   ├── row_comparator.rs
│   ├── key_comparator.rs
│   ├── key_builder.rs
│   ├── duplicate_detector.rs
│   ├── difference.rs
│   └── summary.rs
│
├── excel/
│   └── exporter.rs
│
├── models/
│   ├── file_info.rs
│   ├── parse_options.rs
│   ├── compare_options.rs
│   ├── compare_result.rs
│   ├── composite_key.rs
│   └── difference.rs
│
└── error/
    └── app_error.rs
```

---

# 64. Data Models

## 64.1 FileInfo

```rust
struct FileInfo {
    path: String,
    encoding: String,
    delimiter: String,
    headers: Vec<String>,
    row_count: Option<u64>,
}
```

---

## 64.2 DelimiterOption

```rust
enum DelimiterOption {
    Auto,
    Comma,
    Semicolon,
    Tab,
    Pipe,
    Colon,
    Custom(char),
}
```

---

## 64.3 EncodingOption

概念：

```rust
enum EncodingOption {
    Auto,
    Utf8,
    Big5,
    Cp950,
    Utf16Le,
    Utf16Be,
}
```

---

## 64.4 ParseOptions

```rust
struct ParseOptions {
    encoding: EncodingOption,
    delimiter: DelimiterOption,
}
```

---

## 64.5 ComparisonMode

```rust
enum ComparisonMode {
    KeyBased,
    RowByRow,
}
```

---

## 64.6 CompareOptions

```rust
struct CompareOptions {
    file_a_path: String,
    file_b_path: String,

    file_a_parse_options: ParseOptions,
    file_b_parse_options: ParseOptions,

    comparison_mode: ComparisonMode,

    key_columns: Vec<String>,
    excluded_columns: Vec<String>,

    trim_whitespace: bool,
    ignore_case: bool,
}
```

---

## 64.7 CompositeKey

```rust
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct CompositeKey(Vec<String>);
```

---

## 64.8 KeyValue

```rust
struct KeyValue {
    column: String,
    value: String,
}
```

---

## 64.9 DifferenceType

```rust
enum DifferenceType {
    ValueChanged,
    AOnly,
    BOnly,
    ColumnAOnly,
    ColumnBOnly,
    DuplicateKeyA,
    DuplicateKeyB,
    EmptyKeyA,
    EmptyKeyB,
    IncompleteKeyA,
    IncompleteKeyB,
}
```

---

## 64.10 Difference

```rust
struct Difference {
    key_values: Vec<KeyValue>,

    row_a: Option<u64>,
    row_b: Option<u64>,

    column_name: Option<String>,

    value_a: Option<String>,
    value_b: Option<String>,

    difference_type: DifferenceType,
}
```

---

## 64.11 CompareResult

```rust
struct CompareResult {
    identical: bool,

    rows_a: u64,
    rows_b: u64,

    key_columns: Vec<String>,
    compared_columns: Vec<String>,
    excluded_columns: Vec<String>,

    matched_records: u64,
    same_records: u64,
    different_records: u64,

    a_only_records: u64,
    b_only_records: u64,

    duplicate_keys_a: u64,
    duplicate_keys_b: u64,

    different_cells: u64,

    differences: Vec<Difference>,
}
```

---

# 65. Tauri Commands

至少：

```text
inspect_file
compare_files
export_excel
cancel_compare
```

概念：

```rust
#[tauri::command]
async fn inspect_file(...)

#[tauri::command]
async fn compare_files(...)

#[tauri::command]
async fn export_excel(...)

#[tauri::command]
async fn cancel_compare(...)
```

---

# 66. Rust Dependencies

建議評估：

```toml
tauri
serde
serde_json
csv
encoding_rs
chardetng
rust_xlsxwriter
thiserror
tokio
```

用途：

```text
tauri
Desktop Framework

csv
CSV / Delimited Text Parsing

encoding_rs
Encoding Conversion

chardetng
Encoding Detection

rust_xlsxwriter
Excel XLSX Export

serde / serde_json
Frontend / Rust Data Transfer

thiserror
Error Handling

tokio
Async / Background Tasks
```

---

# 67. Local-only Security

本工具必須：

```text
100% Local Processing
```

不得：

```text
Upload File
Send API
Send Telemetry
Send Analytics
Send Data to Cloud
```

不需：

```text
Login
Account
API Key
Internet
Database
Server
```

---

# 68. Logging

允許紀錄：

```text
Application Start
File Path
File Open Result
Detected Encoding
Detected Delimiter
Header Count
Record Count
Compare Mode
Key Column Names
Excluded Column Names
Compare Duration
Difference Count
Error Code
```

不得紀錄：

```text
完整資料列
客戶姓名
身分證字號
帳號
交易資料
實際欄位值
```

若未來要提供 Debug Log，敏感資料仍不得直接寫入。

---

# 69. Portable Settings

若需保存使用者設定，可使用：

```text
程式目錄/config.json
```

或：

```text
%APPDATA%/CSVCompare/
```

建議可保存：

- 最近一次視窗大小
- 最近一次輸出資料夾
- UI Preference

不應預設保存：

- 最近開啟 CSV 的完整內容
- 欄位資料值
- 比對結果明細

---

# 70. V1 Acceptance Criteria

## Desktop

- [ ] Tauri 2
- [ ] Rust
- [ ] Windows 10 64-bit
- [ ] Windows 11 64-bit
- [ ] Portable
- [ ] 無需 Administrator
- [ ] 無需 Server
- [ ] 無需 Database
- [ ] 無需 Internet

## File

- [ ] `.csv`
- [ ] `.txt`
- [ ] File Picker
- [ ] 手動輸入 Path
- [ ] File A / File B 可使用不同格式

## Encoding

- [ ] UTF-8
- [ ] UTF-8 BOM
- [ ] Big5
- [ ] CP950
- [ ] UTF-16 LE
- [ ] UTF-16 BE
- [ ] Auto Detection
- [ ] Manual Override

## Delimiter

- [ ] Comma
- [ ] Semicolon
- [ ] Tab
- [ ] Pipe
- [ ] Colon
- [ ] Custom Single Character
- [ ] Auto Detection
- [ ] Manual Override
- [ ] File A / B 可使用不同 Delimiter

## Parsing

- [ ] 支援 `"編號","ID"`
- [ ] 支援 `"123","ABC"`
- [ ] 支援全部欄位 Quote
- [ ] 支援部分欄位 Quote
- [ ] 支援 Delimiter inside quoted field
- [ ] 支援 escaped quote
- [ ] 支援 newline inside quoted field
- [ ] Quote 不可成為 Value
- [ ] 正確處理 Header

## Header

- [ ] 依 Header Name Mapping
- [ ] Column Order 不同仍可比對
- [ ] Common Columns
- [ ] A Only Columns
- [ ] B Only Columns
- [ ] Duplicate Header Error

## Key

- [ ] 單一 Key
- [ ] Composite Key
- [ ] Key 只允許共同欄位
- [ ] Key 欄位不可排除
- [ ] Key 使用 String Strict Compare
- [ ] Row Order 不影響 Key-based Compare
- [ ] Empty Key Detection
- [ ] Incomplete Key Detection
- [ ] Duplicate Key A Detection
- [ ] Duplicate Key B Detection
- [ ] Duplicate Key 不可偷偷覆寫

## Compare

- [ ] Key-based Compare
- [ ] Row-by-Row Compare
- [ ] Excluded Columns
- [ ] Value Changed
- [ ] A Only
- [ ] B Only
- [ ] Column A Only
- [ ] Column B Only
- [ ] Strict String Compare
- [ ] 保留前導零
- [ ] 可選 Ignore Trim
- [ ] 可選 Ignore Case

## Result

- [ ] Summary
- [ ] Difference Table
- [ ] Key 顯示
- [ ] Row A
- [ ] Row B
- [ ] Column
- [ ] File A Value
- [ ] File B Value
- [ ] Difference Type
- [ ] Search
- [ ] Filter

## Excel

- [ ] XLSX
- [ ] Summary
- [ ] Differences
- [ ] Column_Differences
- [ ] Excluded_Columns
- [ ] Duplicate_Keys
- [ ] Key 欄位分欄輸出
- [ ] 原始值用 Text Format
- [ ] 保留前導零
- [ ] Freeze Header
- [ ] Auto Filter

## Performance

- [ ] 至少可處理 100 MB + 100 MB
- [ ] Streaming Reader
- [ ] Streaming Decoder
- [ ] Streaming CSV Parser
- [ ] UI 不凍結
- [ ] 顯示 Progress
- [ ] 支援 Cancel

## Security

- [ ] Local Only
- [ ] 無 Upload
- [ ] 無 API
- [ ] 無 Telemetry
- [ ] Log 不含資料內容

---

# 71. 最終整合案例

File A：

```csv
"客戶編號","交易日期","交易序號","金額","更新時間"
"001","2026/09/18","01","1000","08:00"
"001","2026/09/18","02","2000","08:00"
"002","2026/09/18","01","3000","08:00"
```

File B：

```text
"交易日期"|"交易序號"|"客戶編號"|"金額"|"更新時間"
"2026/09/18"|"01"|"002"|"3000"|"09:30"
"2026/09/18"|"01"|"001"|"1000"|"09:30"
"2026/09/18"|"02"|"001"|"2500"|"09:30"
```

設定：

```text
File A Encoding：
CP950

File A Delimiter：
Comma

File B Encoding：
UTF-8

File B Delimiter：
Pipe

Compare Mode：
Key-based

Key：
客戶編號
交易日期
交易序號

Excluded Columns：
更新時間
```

系統應先將：

```text
File A
Comma-separated
```

及：

```text
File B
Pipe-separated
```

解析為同一邏輯資料結構。

Column Order 不同：

```text
不影響比較
```

Row Order 不同：

```text
不影響 Key-based Compare
```

更新時間不同：

```text
因為屬於 Excluded Column，
不影響比較結果。
```

最後只發現：

```text
Key：
客戶編號 = 001
交易日期 = 2026/09/18
交易序號 = 02

Column：
金額

File A：
2000

File B：
2500

Type：
VALUE_CHANGED
```

Summary：

```text
❌ 資料內容不同

Matched Records：3
Same Records：2
Different Records：1
A Only：0
B Only：0
Different Cells：1
Duplicate Keys A：0
Duplicate Keys B：0
```

並可匯出：

```text
CSV_Compare_Result.xlsx
```

---

# 72. Definition of Done

使用者可直接執行：

```text
CSVCompare.exe
```

選擇兩個來源檔案。

例如：

File A：

```csv
"編號","ID"
"123","ABC"
```

File B：

```text
"ID"|"編號"
"ABC"|"123"
```

設定：

```text
Key：
編號
```

程式應：

1. 自動 / 手動辨識 Encoding。
2. 自動 / 手動辨識 Delimiter。
3. 正確解析 Quote。
4. 依 Header Name 建立 Column Mapping。
5. 依 Key 找到對應 Record。
6. 忽略 Column Order。
7. 忽略 Row Order。
8. 比較所有非排除欄位。
9. 顯示 Summary。
10. 顯示 Difference。
11. 可匯出 XLSX。
12. 全流程完全在本機完成。

上述條件全部完成，即視為 V1 Done。
