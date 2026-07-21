# SQL Server Detection Rules Reference

## Overview

This document lists all detection rules defined in `detection_rules.yaml` for Microsoft SQL Server context detection. Detection rules trigger the skill pack when SQL Server-related activity is identified.

## Browser URL Detection Rules

### Rule 1: SSMS Browser Detection
- **ID:** mssql-detect-browser-ssms
- **Pattern:** `sql-server|ssms|sqlservermanagementstudio|azure-data-studio|mssql`
- **Confidence:** 0.85
- **Priority:** 9
- **Description:** Detects SQL Server Management Studio browser interface or remote access

### Rule 2: Azure Data Studio Browser
- **ID:** mssql-detect-browser-ads
- **Pattern:** `azure.data.studio|ads.azure.com|github.com/microsoft/azure-data-studio`
- **Confidence:** 0.90
- **Priority:** 9
- **Description:** Detects Azure Data Studio interface

## Window Title Detection Rules

### Rule 3: SSMS Window Title
- **ID:** mssql-detect-window-ssms
- **Pattern:** `(?i)sql server management studio|ssms|microsoft sql server`
- **Confidence:** 0.95
- **Priority:** 10
- **Description:** Detects SSMS window titles

### Rule 4: Azure Data Studio Window
- **ID:** mssql-detect-window-ads
- **Pattern:** `(?i)azure data studio|ads - `
- **Confidence:** 0.95
- **Priority:** 9
- **Description:** Detects Azure Data Studio window titles

## CLI Command Detection Rules

### Rule 5: sqlcmd
- **ID:** mssql-detect-cli-sqlcmd
- **Pattern:** `^sqlcmd\b`
- **Confidence:** 0.95
- **Priority:** 10

### Rule 6: bcp
- **ID:** mssql-detect-cli-bcp
- **Pattern:** `^bcp\b`
- **Confidence:** 0.95
- **Priority:** 10

### Rule 7: sqlpackage
- **ID:** mssql-detect-cli-sqlpackage
- **Pattern:** `^sqlpackage\b`
- **Confidence:** 0.90
- **Priority:** 9

### Rule 8: dbatools
- **ID:** mssql-detect-cli-dbatools
- **Pattern:** `dbatools|dbatools\.io|Get-Dba|Set-Dba|Test-Dba|Invoke-Dba`
- **Confidence:** 0.90
- **Priority:** 9

### Rule 9: PowerShell SQL Module
- **ID:** mssql-detect-cli-powershell
- **Pattern:** `Invoke-Sqlcmd|Sqlcmd|SqlProvider|sqlserver.*powershell`
- **Confidence:** 0.85
- **Priority:** 8

### Rule 10: DacFx
- **ID:** mssql-detect-cli-dacfx
- **Pattern:** `dacpac|dacfx|dbpro|sqlproj`
- **Confidence:** 0.80
- **Priority:** 7

### Rule 11: sqlserver CLI
- **ID:** mssql-detect-sqlserver-cli
- **Pattern:** `^sqlserver\b`
- **Confidence:** 0.95
- **Priority:** 10

## Text Pattern Detection Rules

### Rule 12: SQL Error Messages
- **ID:** mssql-detect-text-error
- **Pattern:** `(?i)\bERROR\b|Msg \d+|Severity \d+`
- **Confidence:** 0.90
- **Priority:** 10
- **Extract:** `Msg (\d+)`
- **Description:** SQL error messages with message numbers and severity levels

### Rule 13: Deadlock Detection
- **ID:** mssql-detect-text-deadlock
- **Pattern:** `(?i)\bdeadlock\b|deadlock-list|victim-process|process-list|resource-list`
- **Confidence:** 0.95
- **Priority:** 10
- **Description:** Deadlock events in terminal output

### Rule 14: Availability Groups
- **ID:** mssql-detect-text-availability-group
- **Pattern:** `(?i)availability\s*group|AG|Always\s*On|hadr|primary\s*replica|secondary\s*replica|failover`
- **Confidence:** 0.92
- **Priority:** 9
- **Description:** Always On availability group events

### Rule 15: TempDB Issues
- **ID:** mssql-detect-text-tempdb
- **Pattern:** `(?i)\btempdb\b|tempdb.*full|tempdb.*wait|tempdb.*allocation|Page latch|PageIO_Latch`
- **Confidence:** 0.92
- **Priority:** 10
- **Description:** TempDB-related issues

### Rule 16: Checkpoint Activity
- **ID:** mssql-detect-text-checkpoint
- **Pattern:** `(?i)checkpoint|lazywriter|dirty\s*page|checkpoint.*age`
- **Confidence:** 0.85
- **Priority:** 8
- **Description:** Checkpoint and lazy writer activity

### Rule 17: Backup Operations
- **ID:** mssql-detect-text-backup
- **Pattern:** `(?i)backup\s*database|backup\s*log|backup\s*completed|backup\s*set|backup\s*device`
- **Confidence:** 0.90
- **Priority:** 9
- **Description:** Backup operations

### Rule 18: Blocking Chains
- **ID:** mssql-detect-text-blocking
- **Pattern:** `(?i)\bblocking\b|blocked\s*process|waittype.*LOCK|latch.*lock|transfer\s*locks`
- **Confidence:** 0.93
- **Priority:** 10
- **Description:** Blocking chain events

### Rule 19: Restore Operations
- **ID:** mssql-detect-text-restore
- **Pattern:** `(?i)restore\s*database|restore\s*header|restore\s*verify|restore\s*from`
- **Confidence:** 0.90
- **Priority:** 9
- **Description:** Database restore operations

### Rule 20: Index Maintenance
- **ID:** mssql-detect-text-index
- **Pattern:** `(?i)ALTER\s+INDEX|CREATE\s+(CLUSTERED|NONCLUSTERED)\s+INDEX|UPDATE\s+STATISTICS|REORGANIZE|REBUILD`
- **Confidence:** 0.90
- **Priority:** 8
- **Description:** Index-related operations

### Rule 21: Memory Pressure
- **ID:** mssql-detect-text-memory
- **Pattern:** `(?i)memory\s*pressure|out\s*of\s*memory|memory\b.*grant|CMEMTHREAD|RESOURCE_SEMAPHORE|buffer\s*pool`
- **Confidence:** 0.90
- **Priority:** 10
- **Description:** Memory pressure events

### Rule 22: Recovery State
- **ID:** mssql-detect-text-recovery
- **Pattern:** `(?i)recovery.*in\s*progress|restoring.*state|recovery.*complete|online.*state`
- **Confidence:** 0.90
- **Priority:** 9
- **Description:** Database recovery state

### Rule 23: DMV Queries
- **ID:** mssql-detect-text-dmv
- **Pattern:** `sys\.dm_(exec|os|tran|db|server|hadr|resource)_\w+`
- **Confidence:** 0.85
- **Priority:** 8
- **Extract:** `sys\.dm_(exec|os|tran|db|server|hadr|resource)_\w+`
- **Description:** Dynamic management view queries

### Rule 24: Wait Statistics
- **ID:** mssql-detect-text-waitstats
- **Pattern:** `(?i)wait.*type|wait.*stats|CXPACKET|SOS_SCHEDULER_YIELD|PAGEIOLATCH|LCK_M_|WAITFOR`
- **Confidence:** 0.88
- **Priority:** 8
- **Description:** Wait statistics analysis queries

### Rule 25: Query Store
- **ID:** mssql-detect-text-querystore
- **Pattern:** `(?i)query\s*store|sys\.query_store|query\s*store.*runtime.*stats|qs_`
- **Confidence:** 0.85
- **Priority:** 8
- **Description:** Query Store operations

### Rule 26: TDE Operations
- **ID:** mssql-detect-text-tde
- **Pattern:** `(?i)transparent\s*data\s*encryption|tde|dmf_db_database_encryption_keys|encryption\s*state`
- **Confidence:** 0.90
- **Priority:** 9
- **Description:** Transparent Data Encryption operations

### Rule 27: Extended Events
- **ID:** mssql-detect-text-extended-events
- **Pattern:** `(?i)CREATE\s+EVENT\s+SESSION|DROP\s+EVENT\s+SESSION|sys\.dm_xe_sessions|sys\.dm_xe_packages`
- **Confidence:** 0.85
- **Priority:** 8
- **Description:** Extended Events session operations

### Rule 28: LSN Sequences
- **ID:** mssql-detect-text-lsn
- **Pattern:** `(?i)log\s*sequence\s*number|LSN|vlf|virtual\s*log\s*file|truncation\s*point`
- **Confidence:** 0.88
- **Priority:** 9
- **Description:** Transaction log sequence operations

## Detection Summary

| Category | Rules | Total |
|----------|-------|-------|
| Browser | 2 | 2 |
| Window Title | 2 | 2 |
| CLI Commands | 7 | 7 |
| Text Patterns | 17 | 17 |
| **Total** | | **28** |

## Usage Notes

- Higher priority rules (10) trigger first
- Rules with extraction capture specific values from output
- Confidence scores help filter false positives
- Text patterns use case-insensitive matching where noted