# MySQL Detection Rules Reference

## Overview

This reference documents all detection rules for identifying MySQL-related context across browser URLs, window titles, CLI commands, and text patterns. Each rule has an ID, name, description, detection type, confidence score, priority, and extraction capability.

## Detection Categories

### CLI Command Detection

| Rule ID | Name | Pattern | Confidence | Priority |
|---------|------|---------|------------|----------|
| mysql-detect-cli-client | MySQL CLI Client | `^mysql\s+` | 0.95 | 10 |
| mysql-detect-mysqladmin | MySQL Admin Tool | `^mysqladmin\s+` | 0.95 | 10 |
| mysql-detect-mysqldump | MySQL Dump Tool | `^mysqldump\s+` | 0.95 | 10 |
| mysql-detect-mysqlpump | MySQL Pump Tool | `^mysqlpump\s+` | 0.90 | 9 |
| mysql-detect-mysqlcheck | MySQL Check Tool | `^mysqlcheck\s+` | 0.90 | 9 |
| mysql-detect-mysqlbinlog | MySQL Binary Log Tool | `^mysqlbinlog\s+` | 0.90 | 9 |
| mysql-detect-mycli | mycli Client | `^mycli\s+` | 0.85 | 8 |
| mysql-detect-mariadb-cli | MariaDB CLI | `^mariadb\s+` | 0.85 | 8 |
| mysql-detect-pt-tools | Percona Toolkit | `^pt-(archiver|osc|...)` | 0.90 | 9 |
| mysql-detect-mysqlsh | MySQL Shell | `^mysqlsh\s+` | 0.90 | 9 |
| mysql-detect-xtrabackup | Percona XtraBackup | `(xtrabackup|innobackupex|xbstream)` | 0.90 | 9 |
| mysql-detect-proxyadmin | ProxySQL Admin | `mysql\s+(-u\s+admin|...)` | 0.85 | 8 |

### Browser URL Detection

| Rule ID | Name | Pattern | Confidence | Priority |
|---------|------|---------|------------|----------|
| mysql-detect-browser-mysql | MySQL Web Admin | `(phpmyadmin|adminer|mysql-workbench)` | 0.90 | 9 |
| mysql-detect-browser-phpmyadmin | phpMyAdmin Browser | `(phpmyadmin|php-my-admin|pma)/` | 0.95 | 9 |
| mysql-detect-browser-adminer | Adminer Interface | `adminer\.(php|sql)` | 0.90 | 8 |

### Window Title Detection

| Rule ID | Name | Pattern | Confidence | Priority |
|---------|------|---------|------------|----------|
| mysql-detect-window-mysql | MySQL Window | `(MySQL|phpMyAdmin|MySQL Workbench)` | 0.85 | 8 |
| mysql-detect-window-mysqlworkbench | MySQL Workbench Window | `(MySQL Workbench|MySQL Modeler)` | 0.90 | 9 |

### Text Pattern Detection — Error Codes

| Rule ID | Name | Pattern | Confidence | Priority |
|---------|------|---------|------------|----------|
| mysql-detect-error-1045 | Authentication Error | `ERROR 1045 (28000)` | 0.98 | 10 |
| mysql-detect-error-1040 | Too Many Connections | `ERROR 1040 (08004)` | 0.98 | 10 |
| mysql-detect-error-2002 | Server Not Available | `ERROR 2002 (HY000)` | 0.98 | 10 |
| mysql-detect-error-1062 | Duplicate Entry | `ERROR 1062 (23000)` | 0.95 | 9 |
| mysql-detect-error-1205 | Lock Wait Timeout | `ERROR 1205 (HY000)` | 0.95 | 10 |
| mysql-detect-error-1050 | Table Already Exists | `ERROR 1050 (42S01)` | 0.95 | 8 |
| mysql-detect-error-1064 | Syntax Error | `ERROR 1064 (42000)` | 0.95 | 8 |
| mysql-detect-error-1091 | Cannot Drop Column | `ERROR 1091 (42000)` | 0.90 | 8 |
| mysql-detect-error-1146 | Table Not Found | `ERROR 1146 (42S002)` | 0.98 | 9 |
| mysql-detect-error-2006 | Server Gone Away | `ERROR 2006 (HY000)` | 0.95 | 10 |
| mysql-detect-error-3159 | Native Password Error | `ERROR 3159 (HY000)` | 0.95 | 9 |
| mysql-detect-error-3546 | InnoDB Inconsistent | `ERROR 3546 (HY000)` | 0.90 | 10 |

### Text Pattern Detection — InnoDB

| Rule ID | Name | Pattern | Confidence | Priority |
|---------|------|---------|------------|----------|
| mysql-detect-innodb-corruption | InnoDB Corruption | `InnoDB.*corrupt|InnoDB.*page checksum` | 0.95 | 10 |
| mysql-detect-innodb-oom | InnoDB OOM | `InnoDB.*out of memory|InnoDB.*buffer pool.*full` | 0.95 | 10 |
| mysql-detect-innodb-status | InnoDB Status Output | `--- INNODB STATUS ---|INSERT BUFFER|BUFFER POOL` | 0.95 | 9 |

### Text Pattern Detection — Replication

| Rule ID | Name | Pattern | Confidence | Priority |
|---------|------|---------|------------|----------|
| mysql-detect-replication-lag | Replication Lag | `replication lag|Slave_SQL_Running.*No` | 0.95 | 10 |
| mysql-detect-replica-status | Replication Status | `SHOW REPLICA STATUS|Slave_IO_State|Last_SQL_Error` | 0.95 | 10 |
| mysql-detect-mysql-binlog | Binary Log | `binlog|binary.*(log|position|format)` | 0.90 | 9 |
| mysql-detect-gtid-executed | GTID Execution | `gtid_executed|gtid_purged|uuid_set` | 0.90 | 9 |

### Text Pattern Detection — Performance

| Rule ID | Name | Pattern | Confidence | Priority |
|---------|------|---------|------------|----------|
| mysql-detect-deadlock | Deadlock Detection | `Deadlock found when trying|ERROR 1213 (40001)` | 0.98 | 10 |
| mysql-detect-slow-query | Slow Query | `Query_time:.*[0-9]{2,}|SLOW_QUERY` | 0.90 | 9 |
| mysql-detect-full-table-scan | Full Table Scan | `(Full table scan|filesort|temporary table)` | 0.85 | 9 |
| mysql-detect-innodb-lock-wait | InnoDB Lock Wait | `lock wait|data_lock_waits|blocking_pid` | 0.95 | 10 |
| mysql-detect-pid-lock | Process List Lock | `SHOW PROCESSLIST|processlist.*state.*Locked` | 0.90 | 9 |

### Text Pattern Detection — Infrastructure

| Rule ID | Name | Pattern | Confidence | Priority |
|---------|------|---------|------------|----------|
| mysql-detect-max-connections | Max Connections | `max_connections|Aborted_connects|Too many connections` | 0.95 | 10 |
| mysql-detect-tablespace-full | Tablespace Full | `(No space left on device|disk full|ENOSPC)` | 0.95 | 10 |
| mysql-detect-my-cnf | MySQL Config | `(my.cnf|my.ini|mysqld.*--defaults|datadir=)` | 0.85 | 8 |
| mysql-detect-ibdata1 | InnoDB Tablespace | `ibdata1|innodb_data_file|system tablespace` | 0.90 | 8 |
| mysql-detect-connection | MySQL Connection | `(mysql|mysqli|PDO.*mysql|jdbc:mysql)` | 0.80 | 7 |

### Text Pattern Detection — Security

| Rule ID | Name | Pattern | Confidence | Priority |
|---------|------|---------|------------|----------|
| mysql-detect-role-creation | MySQL Role Management | `(CREATE ROLE|DROP ROLE|GRANT.*TO.*ROLE)` | 0.85 | 8 |

## Usage

Each detection rule is evaluated against terminal output, browser URLs, window titles, and text patterns in the following order:

1. **CLI Command Detection** — Highest confidence for active MySQL work
2. **Browser URL Detection** — Indicates MySQL admin tools in use
3. **Window Title Detection** — Indicates MySQL application context
4. **Error Pattern Detection** — Identifies specific failure modes
5. **Text Pattern Detection** — Broader context indicators

Confidence scores reflect the reliability of the detection. Higher scores indicate more certain matches. Priority scores determine which rules are evaluated first when multiple patterns could match.