# SQL Server Terminology Glossary

## Overview

This glossary provides definitions of key SQL Server terms essential for enterprise database administration and engineering.

## A

- **ACID** — Atomicity, Consistency, Isolation, Durability — the four properties that guarantee reliable transaction processing
- **ADR** — Accelerated Database Recovery — SQL Server 2019+ feature that uses persistent versioning to improve recovery time
- **AG** — Availability Group — SQL Server high availability solution (Always On)
- **B-tree** — Balanced tree data structure used for index implementation
- **BCP** — Bulk Copy Program — SQL Server utility for bulk data transfer
- **BPE** — Buffer Pool Extension — SSD-based extension of the buffer pool (deprecated in SQL Server 2017+)
- **Buffer Pool** — In-memory cache for data pages
- **Bulk-Logged Recovery** — Recovery model that minimally logs bulk operations

## C

- **CCK** — Common Criteria Key — encryption key for Always Encrypted
- **CEK** — Column Encryption Key — key used to encrypt column data
- **Checkpoint** — Process that flushes dirty pages from buffer pool to disk
- **CI** — Clustered Index
- **CMK** — Column Master Key — key that protects column encryption keys
- **CMEMTHREAD** — Wait type indicating contention on memory-allocated threads
- **CO** — Cluster Operator — in OpenShift context; in SQL Server context, refers to Cluster Operator for Always On
- **Columnstore Index** — Index that stores data in columnar format for analytical queries
- **Concurrency** — Ability to process multiple transactions simultaneously
- **Connectivity** — Network protocol configuration for client connections
- **Contained Database** — Database that contains all metadata and configuration needed to run independently
- **Cost Model** — SQL Server optimizer's estimation system for plan selection
- **CTE** — Common Table Expression — temporary result set within a query
- **CU** — Cumulative Update — SQL Server cumulative patch
- **DACPAC** — Deployable Application Component PAC — XML representation of database schema
- **DAC** — Dedicated Administrator Connection — emergency administrative connection
- **Data File** — .mdf or .ndf file storing table and index data
- **DEK** — Database Encryption Key — key used for TDE
- **DF** — Default File — primary data file (.mdf)
- **DMF** — Dynamic Management Function — system function for monitoring
- **DMV** — Dynamic Management View — system view for monitoring
- **Differential Backup** — Backup of pages changed since last full backup
- **DLM** — Distributed Lock Manager — manages locks in distributed environments
- **DNQ** — Did Not Qualify — operator returns no rows

## E

- **ECP** — External Certificate Provider — certificate management for encryption
- **EKM** — Extensible Key Management — external key storage (HSM)
- **ENDPOINT** — Network endpoint for Always On replicas
- **Extent** — 64 KB (8 consecutive pages) allocation unit
- **External Script** — Python or R code execution in SQL Server
- **Extended Events** — Lightweight performance monitoring system

## F

- **FCK** — Failover Cluster Key — key for FCI configuration
- **FCI** — Failover Cluster Instance — Windows Server failover cluster for SQL Server
- **Filestream** — Feature storing BLOB data on file system while tracking in database
- **Fill Factor** — Index leaf page fill percentage
- **First Chance Exception** — Exception caught by runtime before reaching application
- **FK** — Foreign Key — referential integrity constraint
- **FOC** — Final Operator in cost — optimizer cost calculation
- **Fragmentation** — Logical ordering loss in indexes
- **Full Recovery** — Recovery model with complete transaction log retention

## G

- **GAM** — Global Allocation Map — tracks extent allocation
- **GC** — Garbage Collection — removal of obsolete row versions
- **GMV** — Global Memory View — memory management view
- **GPU** — Graphics Processing Unit — used in SQL Server 2019+ for batch mode

## H

- **HA** — High Availability — system uptime and disaster recovery
- **HADR** — High Availability and Disaster Recovery — Always On feature set
- **Hash Join** — Join algorithm using hash tables for equality joins
- **Heap** — Table without clustered index
- **HSM** — Hardware Security Module — external key management device

## I

- **IAM** — Index Allocation Map — tracks extent allocation for objects
- **IQP** — Intelligent Query Processing — SQL Server 2017+ optimizer features
- **I/O** — Input/Output — disk operations
- **IS** — Intent Shared — lock type for hierarchy locking
- **IX** — Intent Exclusive — lock type for hierarchy locking
- **IU** — Intent Update — lock type for hierarchy locking

## L

- **LCK_M_* ** — Lock wait types (LCK_M_S, LCK_M_X, LCK_M_U, etc.)
- **Latch** — Short-duration internal lock for memory structures
- **Lazy Writer** — Background thread that flushes cold pages from buffer pool
- **Lease** — Ownership period for distributed resources
- **LOB** — Large Object — varchar(max), nvarchar(max), varbinary(max), xml, etc.
- **Lock Manager** — Component managing all locks
- **Logical CPU** — Processor core available to SQL Server
- **LSB** — Log Sequence Beginning — first log record in VLF
- **LSN** — Log Sequence Number — unique identifier for log record
- **LTRIM/RTRIM** — String functions to remove leading/trailing spaces

## M

- **MAXDOP** — Maximum Degree of Parallelism — parallel execution limit
- **MDL** — Memory Dependency List — memory tracking for queries
- **ME** — Memory Extension — deprecated term for BPE
- **Memory Clerk** — Component responsible for a type of memory allocation
- **MI** — Managed Instance — Azure SQL Managed Instance
- **MM** — Memory Manager — component managing memory allocation
- **MSDB** — System database storing job and backup metadata
- **Multi-Subnet** — Deployment across multiple network subnets

## N

- **NCI** — Non-Clustered Index
- **NOCHECK** — Disable constraint checking
- **NORECOVERY** — Restore option keeping database in restoring state
- **NULLEnable** — Column property for null handling

## O

- **OLE DB** — Object Linking and Embedding, Database — data access API
- **OLTP** — Online Transaction Processing — transactional workload
- **OLAP** — Online Analytical Processing — analytical workload
- **Online Index** — Index operation that keeps table available
- **OQ** — Optimized Query — query plan with minimal cost

## P

- **Page** — 8 KB fundamental allocation unit
- **PAGEIOLATCH** — Wait type for I/O latch on data pages
- **PAGELATCH** — Wait type for in-memory latch on data pages
- **PFS** — Page Free Space — tracks page utilization
- **PID** — Process ID — Windows process identifier
- **PITR** — Point-in-Time Recovery — restore to specific moment
- **PK** — Primary Key — unique identifier for table rows
- **Plan Cache** — In-memory cache of compiled execution plans
- **Plan Guide** — Mechanism to influence plan selection without changing query
- **Policy-Based Management** — SQL Server policy evaluation framework
- **PolyBase** — Feature for external data access
- **Primary File** — .mdf file (contains system tables)

## Q

- **QEP** — Query Execution Plan — plan chosen by optimizer
- **Query Store** — Feature capturing query performance history
- **QSM** — Query Store Memory — Query Store memory usage

## R

- **RAD** — Recovery Advisor Database
- **RAID** — Redundant Array of Independent Disks
- **RDBMS** — Relational Database Management System
- **Recovery Model** — Database property controlling log retention (Full, Bulk-Logged, Simple)
- **Reorganize** — Index maintenance operation for moderate fragmentation
- **Rebuild** — Index maintenance operation for heavy fragmentation
- **RID** — Row Identifier — pointer to data row in heap
- **RLS** — Row-Level Security — security feature for row access control
- **RPO** — Recovery Point Objective — maximum acceptable data loss
- **RTO** — Recovery Time Objective — maximum acceptable downtime

## S

- **SAN** — Storage Area Network
- **SB** — System Base — base for system object IDs
- **SGAM** — Shared Global Allocation Map — tracks mixed extent pages
- **SSB** — SQL Server Base — base for SQL Server system objects
- **SSMS** — SQL Server Management Studio — primary admin tool
- **SSRS** — SQL Server Reporting Services
- **SSAS** — SQL Server Analysis Services
- **SSIS** — SQL Server Integration Services
- **SSN** — Social Security Number — PII category
- **Startup Parameter** — SQL Server configuration at startup
- **Stolen Page** — Page allocated to a memory clerk
- **Stream Aggregate** — Aggregation without pre-sorting
- **SYNCH** — Synchronization — transaction synchronization mechanism

## T

- **T-DML** — Transactional DML — data manipulation language
- **T-SQL** — Transact-SQL — Microsoft's SQL extension
- **TAI** — Transparent Application Failover
- **TDE** — Transparent Data Encryption — encryption at rest
- **TLI** — Thread Local Information
- **TLM** — Transaction Log Manager
- **TPC** — Transaction Processing Council
- **TRT** — Target Recovery Time — checkpoint interval target
- **TDS** — Tabular Data Stream — client-server protocol
- **Thread** — Execution unit in SQL Server
- **TO** — Transaction Outcome — commit or rollback result
- **TOP** — TOP N query operator

## U

- **UFN** — User-Defined Function (scalar)
- **UDF** — User-Defined Function (table-valued or scalar)
- **UI** — User Interface
- **ULN** — Unique Local Name
- **UM** — User Mode
- **UNDO** — Rollback of uncommitted transactions during recovery
- **UPD** — Update — lock type
- **UQ** — Unique Constraint
- **URL** — Uniform Resource Locator (Azure blob storage)

## V

- **VLF** — Virtual Log File — logical division of transaction log
- **VM** — Virtual Machine
- **VM** — Virtual Memory
- **VO** — Version Owner — row version tracking
- **VS** — Version Store — TempDB space for row versions

## W

- **WAIT** — Waiting — resource wait state
- **WF** — Work Function
- **WKB** — Well-Known Binary — spatial data format
- **WKT** — Well-Known Text — spatial data format
- **Worker Thread** — Thread executing user queries
- **WLM** — Workload Management — resource governor feature
- **WPF** — Windows Presentation Foundation
- **WS** — Windows Server

## X

- **XE** — Extended Events
- **XML** — eXtensible Markup Language
- **XLOCK** — Exclusive lock type
- **XPN** — Extended Procedure

## Z

- **ZOMBIE** — Terminated process with pending cleanup