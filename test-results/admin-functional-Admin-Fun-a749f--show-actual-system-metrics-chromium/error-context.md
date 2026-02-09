# Page snapshot

```yaml
- generic [active] [ref=e1]:
  - navigation [ref=e2]:
    - generic [ref=e4]:
      - generic [ref=e5]:
        - generic [ref=e6]: MCP Context Browser
        - generic [ref=e7]: Admin
      - generic [ref=e8]:
        - link "Dashboard" [ref=e9] [cursor=pointer]:
          - /url: /
        - link "Config" [ref=e10] [cursor=pointer]:
          - /url: /ui/config
        - link "Health" [ref=e11] [cursor=pointer]:
          - /url: /ui/health
        - link "Indexing" [ref=e12] [cursor=pointer]:
          - /url: /ui/indexing
        - link "Browse" [ref=e13] [cursor=pointer]:
          - /url: /ui/browse
  - main [ref=e14]:
    - generic [ref=e15]:
      - generic [ref=e16]:
        - heading "Health Status" [level=1] [ref=e17]
        - paragraph [ref=e18]: System health and dependency status
      - button "Refresh" [ref=e19] [cursor=pointer]
    - generic [ref=e20]: "{\"status\":\"healthy\",\"uptime_seconds\":2,\"active_indexing_operations\":0,\"dependencies\":[{\"name\":\"embedding_provider\",\"status\":\"Unknown\",\"message\":\"Total queries: 0, Failed: 0\",\"latency_ms\":0,\"last_check\":1770657765},{\"name\":\"vector_store\",\"status\":\"Healthy\",\"message\":\"Active indexing operations: 0\",\"latency_ms\":null,\"last_check\":1770657765},{\"name\":\"cache\",\"status\":\"Unknown\",\"message\":\"Cache hit rate: 0.0%\",\"latency_ms\":null,\"last_check\":1770657765}],\"dependencies_status\":\"Healthy\"}"
    - generic [ref=e21]:
      - generic [ref=e22]:
        - heading "Readiness Probe" [level=2] [ref=e23]
        - generic [ref=e24]: "{\"ready\":true}"
      - generic [ref=e25]:
        - heading "Liveness Probe" [level=2] [ref=e26]
        - generic [ref=e28]: ALIVE
  - contentinfo [ref=e29]:
    - generic [ref=e30]: MCP Context Browser v0.1.5 | Admin Panel
```