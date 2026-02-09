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
        - button "Auto" [ref=e14] [cursor=pointer]
  - main [ref=e15]:
    - generic [ref=e16]:
      - heading "Browse Indexed Code" [level=1] [ref=e17]
      - paragraph [ref=e18]: Explore collections, files, and code chunks
    - generic [ref=e19]:
      - generic [ref=e20]:
        - heading "Collections" [level=2] [ref=e21]
        - button "Refresh" [ref=e22] [cursor=pointer]
      - paragraph [ref=e25]: Loading...
  - contentinfo [ref=e26]:
    - generic [ref=e27]: MCP Context Browser v0.1.5 | Browse Indexed Code
```