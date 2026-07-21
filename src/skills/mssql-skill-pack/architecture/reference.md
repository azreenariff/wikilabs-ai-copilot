# MSSQL Architecture Reference

## Overview

This document provides architectural details for Microsoft SQL Server deployments.

## Core Architecture

### Components

1. **SQL Server Engine**: Relational database engine
2. **Query Processor**: Query compilation, optimization, execution
3. **Storage Engine**: Pages, extents, data files
4. **Transaction Log**: Write-ahead logging
5. **Buffer Pool**: Memory cache for data pages
6. **Lock Manager**: Transaction concurrency control
7. **Replication Services**: Data distribution
8. **Full-Text Search**: Text indexing and search

### Memory Architecture

```
SQL Server Memory
├── Buffer Pool (Data Cache)
│   ├── In-Memory Data Pages
│   ├── Procedure Cache
│   └── Plan Cache
├── Lock Manager
├── Free Space
└── Stolen Memory
    ├── Extensions (CLR, XEvents)
    └── External Consumers
```

### Storage Architecture

```
SQL Server
├── Data Files (.mdf)
│   ├── Primary Filegroup
│   ├── Secondary Filegroups
│   └── FileStream Filegroups
├── Transaction Log Files (.ldf)
│   ├── VLFs (Virtual Log Files)
│   └── Log Chain
└── Temporary Database (tempdb)
    ├── Data Files
    ├── Transaction Log
    └── Version Store
```

### SQL Server Versions

| Version | Release | Features |
|---------|---------|----------|
| **2017** | 2017 | Linux support, Intelligent Query Processing |
| **2019** | 2019 | Graph tables, Big Data Clusters |
| **2022** | 2022 | Cryptographic operations, Accelerated DB Recovery |

## Scale Architecture

| Scale | CPU | Memory | Storage | Use Case |
|-------|-----|--------|---------|----------|
| Small | 4-8 cores | 16-32GB | 500GB | Development, small business |
| Medium | 8-16 cores | 32-64GB | 1-2TB | Production, mid-size business |
| Large | 16-32 cores | 64-128GB | 2-10TB | Enterprise, high traffic |
| Enterprise | 32+ cores | 128GB+ | 10TB+ | Mission-critical, data warehouse |

## References

- SQL Server Architecture: https://learn.microsoft.com/en-us/sql/
- SQL Server Internals: https://learn.microsoft.com/en-us/sql/
- SQL Server Components: https://learn.microsoft.com/en-us/sql/