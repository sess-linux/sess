# `sess prune`

```
sess prune
```

Scans the storage directory and removes any saved session file that's
corrupt or fails to parse, instead of leaving it to break `sess list`
silently. Reports how many files were removed.
