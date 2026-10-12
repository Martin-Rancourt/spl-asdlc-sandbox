# The end-to-end scenario

Two parts, run in order against the **staging** bridge (the latest `main` of spl-asdlc): (a) a ticket goes from Ready to merged and To Deploy; (b) a release cycle ships it and closes it Done. Each step gives the gesture, what the bridge must do, and what to check. A step whose expected behaviour does not happen is a finding against spl-asdlc's `main`: note it, with the tick log lines, before going further.

The gestures by hand and what follows each are spl-asdlc's `docs/setup/manual-gestures.md`; the statuses are its `docs/spec/03-vocabulary.md`.

## Before the first run

Once per sandbox, by the owner:

- The GitHub App is installed on this repository, and the secrets `SPL_ASDLC_APP_ID` and `SPL_ASDLC_APP_PRIVATE_KEY` are set (spl-asdlc's `docs/setup/github-app.md`).
- The rulesets of `docs/setup/repository.md`: `main` and `release/**` require a PR and the status check `required-check`, block force pushes, the App on the bypass list; a tag ruleset blocks updates and deletions on `refs/tags/v*` (the candidates and the releases).
- The tracker's GitHub integration sees this repository, so that a branch or PR named after a ticket is linked to it (`[tracker] link_fallback` is `none`).
- On the bridge's host, the sandbox instance: its own directory with a clone of this repository (pulled before each tick), the **staging** binary, its timer, the existing credentials, and the instance overlay `spl-asdlc.instance.toml` beside the clone's `spl-asdlc.toml`:

  ```toml
  schema = 1

  [product]
  repository = "<owner>/spl-asdlc-sandbox"

  [owner]
  tracker_user = "<owner as the tracker knows them>"

  [tracker]
  team = "<TEAM>"
  service_identity = "<the bridge's service account>"
  team_keys = ["<TEAM>"]

  [release]
  release_please_authors = ["app/<app slug>"]
  ```

The shell variables the commands below use:

```sh
REPO=<owner>/spl-asdlc-sandbox
TEAM=<TEAM>                       # the tracker team key of the overlay
P=spl-asdlc-sandbox               # [product] name
# On the bridge's host:
CFG=<the instance's clone>/spl-asdlc.toml
LOG=<the instance's tick log>     # its --log, by default $XDG_STATE_HOME/spl-asdlc/<owner>-spl-asdlc-sandbox.jsonl
```

Check the instance before each run, on the host:

```sh
spl-asdlc --version                                   # the staging build
gh release view staging --repo <owner>/spl-asdlc      # the commit it was built from
spl-asdlc config check --file "$CFG"
spl-asdlc doctor --file "$CFG"                        # every line ok; "PR review: ok — none"; the release checks ok
```

The bridge ticks every five minutes on its timer. To see what the next tick will do without waiting: `spl-asdlc tick --file "$CFG" --dry-run`. To read what it did: `tail -n 20 "$LOG" | jq -c '{when, object, action, outcome}'`.

### Linear from the shell

The tracker gestures below use Linear's GraphQL API with a personal key of the person running the scenario (never the bridge's service key); the same gestures can be made in the Linear app.

```sh
lin() { # lin '<query>' '<variables as JSON>'
  jq -n --arg q "$1" --argjson v "$2" '{query: $q, variables: $v}' |
    curl -sS https://api.linear.app/graphql \
      -H "Authorization: $LINEAR_API_KEY" -H 'Content-Type: application/json' -d @-
}
state_id() { # state_id <status name>
  lin 'query($k: String!) { teams(filter: {key: {eq: $k}}) { nodes { states { nodes { id name } } } } }' \
    "$(jq -n --arg k "$TEAM" '{k: $k}')" |
    jq -r --arg n "$1" '.data.teams.nodes[0].states.nodes[] | select(.name == $n) | .id'
}
show() { # show <identifier>
  lin 'query($id: String!) { issue(id: $id) { identifier state { name } archivedAt labels { nodes { name } } } }' \
    "$(jq -n --arg id "$1" '{id: $id}')" | jq -c .data.issue
}
```

## (a) A ticket, its PR, To Deploy

### 1. Write the ticket and make it Ready

Write the contract (spl-asdlc's `docs/setup/templates/ticket.md`) and judge it:

```sh
cat > ticket.md <<'EOF'
## Goal
The sandbox can say goodbye: `farewell(name)` mirrors `greet`.

## Non-goals
- Any other change to `greet`.

## Acceptance criteria
- [ ] `farewell("sandbox")` returns `Goodbye, sandbox!`, with a unit test.
- [ ] A blank name says goodbye to the world, with a unit test.

## Business rules
None.

## Release note
The sandbox says goodbye.
EOF
spl-asdlc ticket check --description ticket.md        # on the host, or any machine with the binary
```

Create it and set it Ready:

```sh
TEAM_ID=$(lin 'query($k: String!) { teams(filter: {key: {eq: $k}}) { nodes { id } } }' \
  "$(jq -n --arg k "$TEAM" '{k: $k}')" | jq -r '.data.teams.nodes[0].id')
lin 'mutation($i: IssueCreateInput!) { issueCreate(input: $i) { issue { identifier url branchName } } }' \
  "$(jq -n --arg t "$TEAM_ID" --rawfile d ticket.md --arg s "$(state_id Ready)" \
     '{i: {teamId: $t, title: "The sandbox says goodbye", description: $d, stateId: $s}}')" | tee issue.json
ID=$(jq -r .data.issueCreate.issue.identifier issue.json)
BRANCH=$(jq -r .data.issueCreate.issue.branchName issue.json)
```

**Expected**: nothing from the bridge yet (Ready is refinement's status). `show "$ID"` → Ready.

### 2. Branch, implement, open the PR

```sh
gh repo clone "$REPO" sandbox && cd sandbox
git switch -c "$BRANCH"
cat >> src/lib.rs <<'EOF'

/// The farewell for `name`; a blank name says goodbye to the world.
pub fn farewell(name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        "Goodbye, world!".to_string()
    } else {
        format!("Goodbye, {name}!")
    }
}

#[cfg(test)]
mod farewell_tests {
    use super::farewell;

    #[test]
    fn says_goodbye_to_a_name() {
        assert_eq!(farewell("sandbox"), "Goodbye, sandbox!");
    }

    #[test]
    fn a_blank_name_says_goodbye_to_the_world() {
        assert_eq!(farewell(" "), "Goodbye, world!");
    }
}
EOF
cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked
git commit -am "feat: the sandbox says goodbye ($ID)"
git push -u origin "$BRANCH"
gh pr create --repo "$REPO" --draft --title "feat: the sandbox says goodbye ($ID)" \
  --body "$(jq -r .data.issueCreate.issue.url ../issue.json)"
```

**Expected, next tick**: the PR is linked to the ticket (the tracker's integration); the bridge sets the ticket **In Progress**. On GitHub, `pr` runs `guards` and `rustfmt` on the draft (clippy and the tests wait for ready), `pr-title` passes.

Check: `show "$ID"` → In Progress; `gh pr checks --repo "$REPO" "$BRANCH"`; the tick log has the transition.

### 3. Tick the boxes, mark the PR ready

The contract gates the merge: every box ticked (the PR delivers the whole ticket).

```sh
sed 's/- \[ \]/- [x]/' ../ticket.md > ../ticket-done.md
lin 'mutation($id: String!, $d: String!) { issueUpdate(id: $id, input: {description: $d}) { success } }' \
  "$(jq -n --arg id "$ID" --rawfile d ../ticket-done.md '{id: $id, d: $d}')"
gh pr ready --repo "$REPO" "$BRANCH"
gh pr checks --repo "$REPO" "$BRANCH" --watch       # required-check green
```

**Expected, next ticks** (no PR review: no round, no approval, `review.angles = []`): the ticket **In Review** while the PR waits, then the bridge **merges the PR**: a squash pinned to the head SHA, as the App, the branch deleted; the ticket goes **To Deploy** (a release cycle is configured, so a merged ticket waits for a release; never Done here). One merge per tick.

Check:

```sh
gh pr view --repo "$REPO" "$BRANCH" --json state,mergedBy,mergeCommit --jq '{state, by: .mergedBy.login, sha: .mergeCommit.oid}'
show "$ID"                                          # To Deploy
spl-asdlc tick --file "$CFG" --dry-run              # on the host: nothing left to do for this PR
```

The tick log holds the merge decision and the transition, each once.

## (b) A release cycle

### 4. Open the cycle

```sh
gh workflow run release-cut.yml --repo "$REPO" --ref main -f action=open
# or, on the host: spl-asdlc release cut --file "$CFG" --product "$P" --open
gh run watch --repo "$REPO" "$(gh run list --repo "$REPO" --workflow release-cut.yml --limit 1 --json databaseId --jq '.[0].databaseId')"
```

**Expected on GitHub, with no gesture**:

1. `release-cut` (open) pushes `release/<YYYY-MM-DD-HH-MM>-utc` from `main` and ends.
2. That push starts `release-please`, which opens its PR into the release branch as the App: title `chore(release/<stamp>-utc): release spl-asdlc-sandbox <version>`, label `autorelease: pending`, the version bumped in `Cargo.toml`, `Cargo.lock`, the manifest and `CHANGELOG.md` (a `feat` since 0.1.0: `0.2.0`). Its `pr` run skips the Rust jobs (guards reports no code) and `required-check` is green.
3. release-please's green run starts `release-cut` again (`workflow_run`), which cuts the first candidate: the tag `v<version>-rc.1` on the release branch.
4. The tag starts `release-gate`: the build, then the suite `test` on the exact candidate. Green. `release-gate-auto-rerun` runs after it and reruns nothing.

```sh
B=$(git ls-remote --heads origin 'release/*' | sed 's#.*refs/heads/##' | tail -n 1)
gh pr list --repo "$REPO" --base "$B" --label "autorelease: pending"
git ls-remote --tags origin 'v*-rc.*'
gh run list --repo "$REPO" --workflow release-gate.yml --limit 1
```

**Expected from the bridge, next ticks**: the **release issue** (its title names `spl-asdlc-sandbox` and the version), the candidate record, the label `release/spl-asdlc-sandbox-v<version>` on the release issue and on the embarked ticket `$ID`; once the gate is green, the **brief** on the release issue (the embarked ticket with its acceptance criteria, no business rule touched, the link to the gate's evidence, and that no QA angle is configured). No angle sub-item, no consolidation item.

```sh
spl-asdlc release status --file "$CFG" --product "$P"   # on the host: the GO and its inputs; exit 1 while there is no GO
show "$ID"                                              # To Deploy, with the release label
```

### 5. GO: the bridge merges the release-please PR

**Expected, next tick** once the gate is green: `release status` reads GO (gate green, no blocker, no QA angle, nothing on the branch since the candidate but bookkeeping), and the bridge **merges the release-please PR** into the release branch, as the App.

Then, on GitHub, with no gesture:

1. The merge pushes the release branch: `release-please` runs again and creates the GitHub **release** `v<version>` as the App (its `release-cut` follow-up finds nothing to cut and ends green, with a notice).
2. The release starts `release-publish`: **verify** (the release-please PR merged by the App, the release equal to `v<version>-rc.1` but for the bookkeeping paths), **publish** (prints `spl-asdlc-sandbox <version>, certified by v<version>-rc.1 at <sha>`), **close** (opens the merge-back PR from the release branch into `main`, waits for its `required-check`, merges it as a merge commit through the App, deletes the release branch).

```sh
gh pr view --repo "$REPO" <release-please PR> --json state,mergedBy --jq '{state, by: .mergedBy.login}'
gh release view --repo "$REPO" "v<version>"
gh run list --repo "$REPO" --workflow release-publish.yml --limit 1
gh pr list --repo "$REPO" --base main --state merged --limit 1     # the merge-back, a merge commit
git ls-remote --heads origin 'release/*'                           # empty
```

### 6. Closure

**Expected, next tick** after the release is published and the merge back merged: the **closure**. The embarked ticket `$ID`, To Deploy with every box ticked, goes **Done**; the release note is posted; the release issue is closed with its summary. Then the **archiving**: the ticket Done, then the release issue last, both archived (searchable and restorable in Linear).

```sh
show "$ID"                                              # Done, archivedAt set
spl-asdlc release status --file "$CFG" --product "$P"   # the cycle is over
spl-asdlc tick --file "$CFG" --dry-run                  # nothing to do
tail -n 30 "$LOG" | jq -c '{when, object, action, outcome}'
```

A second tick writes nothing (every write is idempotent): the tick log gets no new line.

## Cleanup

A green run leaves nothing open: no PR, no release branch, the ticket and the release issue archived. The tags (`v<version>-rc.1`, `v<version>`) and the release stay: the tag ruleset makes them immutable, and the next run releases the next version. A later run takes another small `feat` or `fix` ticket of the same shape (a function and its unit tests).

A run stopped midway:

```sh
gh pr close --repo "$REPO" <n> --delete-branch                     # an open delivery PR
spl-asdlc release abandon --file "$CFG" --product "$P"             # on the host: a cycle in flight
lin 'mutation($id: String!, $s: String!) { issueUpdate(id: $id, input: {stateId: $s}) { success } }' \
  "$(jq -n --arg id "$ID" --arg s "$(state_id Canceled)" '{id: $id, s: $s}')"
cd .. && rm -rf sandbox ticket.md ticket-done.md issue.json
```

`release abandon` keeps the tags, merges the release branch back into `main`, cancels the release issue and turns what remains into tickets.
