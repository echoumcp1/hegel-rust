---
name: create-pr
description: "Create a pull request for the current branch. Use this whenever you need to open a PR, push changes for review, or the user asks you to create a pull request. Handles rebasing, RELEASE.md, draft creation, CI watching, and copilot review."
---

# Create Pull Request

## Before creating the PR

1. Run the `self-review` skill first. Fix anything it finds.

2. Fetch and rebase on `origin/main`:
   ```bash
   git fetch origin main
   git rebase origin/main
   ```

3. Check if `RELEASE.md` exists. If source code was changed and there is no `RELEASE.md`, create one following `RELEASE-sample.md` and `.claude/changelog-guidance.md`.

4. Review `git diff origin/main...HEAD` to understand the full scope of changes. Write the PR content based on this, not just the latest commit.

## Create the PR

```bash
gh pr create --draft --title "<title>" --body "$(cat <<'EOF'
<body>
EOF
)"
```

- **Title**: short, clear, under 70 characters.
- **Body**: one or two short paragraphs explaining what changed and why. First person, casual tone matching the project's existing PR style. Describe user impact, not implementation details.
- Do not include AI attribution, checklists, test plans, or TODO sections.
- Request a copilot review: `gh pr edit <number> --add-reviewer copilot`

## Watch CI and address feedback

Loop up to three times:

1. Watch CI: `gh pr checks <number> --repo <owner>/<repo> --watch --fail-fast`
   - Note: checks may take a few seconds to register after push. If `--watch` reports "no checks", retry once.
2. If the build fails, read the failure logs, fix, push, and repeat from step 1.
3. For copilot review comments: either fix the issue or resolve the thread if copilot is wrong. Use the GraphQL `resolveReviewThread` mutation to resolve threads.
   - Copilot is fallible. Verify factual claims ("X is deprecated", "Y will fail lint") against actual docs or by running the relevant tool. Don't accept claims at face value.
4. If you made changes, push and repeat from step 1.

After the final green build, wait up to 30 seconds to see if copilot posts any new review comments, then address those if present.
