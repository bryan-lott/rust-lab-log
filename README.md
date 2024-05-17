# Rust Lab Log

A human-readable log of activities taken by the user.

Inspired by the hand written chemistry logs of activity that I saw whenever I visited my dad's lab when I was a kid and the logs we want software to spit out.

## Example

```shell
rlg "testing the lab logger"
```

```markdown
---
id: rlg
aliases: []
tags: []
---

# Rust Lab Log

## 2024

### 2024-01-10

- 2024-01-10 07:48:56 How did we get into this position in the first place? Like, how did we originally corrupt events. IIRC we were using `flask.request` and throughout all my testing it has shown to be consistent.
- 2024-01-10 07:57:54 Was I using `flask.request` to set `self.current_request` and _that_ was getting corrupted? Like, it wasn't `flask.request` in the first place? Does that mean I can replace `self.current_request` with `flask.request` everywhere and still use `@lru_cache`? Seems risky. Create tickets for fixing the application code and figuring out how to cache redis connections.
- 2024-01-10 12:32:19 Combined 2 cards related to caching and removed the update to <redacted> as it's not technically needed.
- 2024-01-10 13:23:59 Currently testing in stage to validate that all of <redacted> continues to work as expected with `@lru_cache` removed from the `load` functions. Hopefully eliminating the threadsafety issue(s)
- 2024-01-10 14:40:00 Threadsafety issue _seems_ to be resolved. However, running into high numbers of redis connections. I'm not sure if these are from the <redacted>. I have validated that we are pulling from the cache for all events except the first.
- 2024-01-10 15:32:42 PR is up and ready for review. Will need to make a minor change to the PR before it goes to production. Removing the "is redis being cached" logging instrumentation. It's not a big deal, but worth cleaning up IMO.

### 2024-01-11

- 2024-01-11 09:24:08 <redacted> pointed out that I was caching the wrong redis connection. I've now cached the other 2 that actually matter and saw a massive drop in the number of connections. Just completed a load test and the results
- 2024-01-11 10:26:46 path forward is to move the functions to the `get_*` pattern and make sure those function returns are cached
- 2024-01-11 10:45:11 Deploying the above change to stage
- 2024-01-11 11:32:09 Reinstalled doom emacs, backed up existing configs, started once to be sure it at least starts up
- 2024-01-11 15:50:18 <redacted> is missing a subscription being created from his app... having a hard time finding the missing piece

### 2024-05-17

- 2025-05-17 07:34:27 testing the lab logger
```

## Configuration
The first time rlg is run it will create a default config file and tell you where it put it. It's recommended to create a symlink to this file to find it easier later.

```toml
default_log_file = "~/Dropbox/notes/rlg.md"
style = "markdown"
```

Note: other styles aren't supported yet, but will be added maybe at some point.
