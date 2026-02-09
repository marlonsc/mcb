# OSV Scanner Warnings - Transitive Dependencies

## Summary
5 WARNING-level advisories from transitive dependencies (not directly fixable).

## Details

### 1. RUSTSEC-2025-0119: number_prefix (unmaintained)
- **Severity**: INFO/WARNING
- **Path**: indicatif → hf-hub → fastembed → mcb-providers
- **Fix**: Wait for fastembed to migrate to `unit-prefix`
- **Impact**: No security vulnerability, just unmaintained

### 2. RUSTSEC-2024-0436: paste (unmaintained)
- **Severity**: INFO/WARNING  
- **Path**: tokenizers/macro_rules_attribute → fastembed → mcb-providers
- **Status**: Author says "finished" not abandoned. Many projects ignore this.
- **Impact**: No security vulnerability

### 3. RUSTSEC-2025-0134: rustls-pemfile (unmaintained)
- **Severity**: INFO/WARNING
- **Path**: 
  - rustls-native-certs → async-nats → mcb-providers
  - tonic → milvus-sdk-rust → mcb-providers
- **Fix**: Wait for async-nats/milvus to migrate to rustls-pki-types 1.9.0+
- **Impact**: No security vulnerability, just unmaintained

### 4. CVE-2023-49092 (RUSTSEC-2023-0071): rsa timing attack
- **Severity**: MEDIUM (5.9 CVSS)
- **Status**: **NO FIX AVAILABLE**
- **Path**: Not found in dependency tree (likely platform-specific or removed)
- **Impact**: Only exploitable over network with timing observation

### 5. RUSTSEC-2023-0089: atomic-polyfill (unmaintained)
- **Severity**: INFO/WARNING
- **Path**: Not found in dependency tree (likely platform-specific or removed)
- **Impact**: No security vulnerability

## Recommendation
**Accept these warnings** as they are:
1. All INFO-level "unmaintained" advisories (not actual vulnerabilities)
2. Transitive dependencies we cannot directly control
3. Require upstream library updates (fastembed, async-nats, milvus-sdk-rust)

## Monitoring
- Track fastembed updates: https://github.com/Anush008/fastembed-rs
- Track async-nats updates: https://github.com/nats-io/nats.rs
- Track milvus-sdk-rust updates: https://github.com/milvus-io/milvus-sdk-rust
