# The end-to-end scenario

Three parts, run in order against the **staging** bridge (the latest `main` of spl-asdlc): (a) a ticket goes from Ready to merged and To Deploy; (b) a release cycle ships it and closes it Done; (c) a PR merged to `main` while a cycle is open appears in the next release's notes. Each step gives the gesture, what the bridge must do, and what to check. A step whose expected behaviour does not happen is a finding against spl-asdlc's `main`: note it, with the tick log lines, before going further.

The gestures by hand and what follows each are spl-asdlc's `docs/setup/manual-gestures.md`; the statuses are its `docs/reference/03-vocabulary.md`.

## Before the first run

Once per sandbox, by the owner:

- The GitHub App is installed on this repository, and the secrets `SPL_ASDLC_APP_ID` and `SPL_ASDLC_APP_PRIVATE_KEY` are set (spl-asdlc's `docs/setup/github-app.md`).
- spl-asdlc's rolling `staging` prerelease holds the Linux binary and its `SHA256SUMS` (its `staging.yml`, on every push to its `main`): the cut's `open` downloads it with the App's token (`spl-asdlc-version: staging`) to run `release prepare`.
- The rulesets of `docs/setup/repository.md`: `main` and `release/**` require a PR and the status check `required-check`, block force pushes, the App on the bypass list; a tag ruleset blocks updates and deletions on `refs/tags/v*` (the candidates and the releases).
- The tracker's GitHub integration sees this repository, so that a branch or PR named after a ticket is linked to it (`[tracker] link_fallback` is `none`).
- On the bridge's host, the sandbox instance: its own directory with a clone of this repository (pulled before each tick), the **staging** binary, its timer, the existing credentials, and the instance overlay `.spl-asdlc/config.instance.toml` beside the clone's `.spl-asdlc/config.toml`:

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
  ```

The shell variables the commands below use:

```sh
REPO=<owner>/spl-asdlc-sandbox
TEAM=<TEAM>                       # the tracker team key of the overlay
P=spl-asdlc-sandbox               # [product] name
# On the bridge's host:
CFG=<the instance's clone>/.spl-asdlc/config.toml
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
# The implementer starts the ticket: it moves it In Progress itself.
lin 'mutation($id: String!, $s: String!) { issueUpdate(id: $id, input: {stateId: $s}) { success } }' \
  "$(jq -n --arg id "$ID" --arg s "$(state_id "In Progress")" '{id: $id, s: $s}')"
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

**Expected, next tick**: the PR is linked to the ticket (the tracker's integration). The PR is a draft, so the bridge writes nothing: the ticket stays **In Progress**, where its implementer put it (the bridge never moves a ticket In Progress for it). On GitHub, `pr` runs `guards` and `rustfmt` on the draft (clippy and the tests wait for ready), `pr-title` passes.

Check: `show "$ID"` → In Progress; `gh pr checks --repo "$REPO" "$BRANCH"`; the tick log has no transition of the ticket.

### 3. Tick the boxes, mark the PR ready

The contract gates the merge: every box ticked (the PR delivers the whole ticket).

```sh
sed 's/- \[ \]/- [x]/' ../ticket.md > ../ticket-done.md
lin 'mutation($id: String!, $d: String!) { issueUpdate(id: $id, input: {description: $d}) { success } }' \
  "$(jq -n --arg id "$ID" --rawfile d ../ticket-done.md '{id: $id, d: $d}')"
gh pr ready --repo "$REPO" "$BRANCH"
gh pr checks --repo "$REPO" "$BRANCH" --watch       # required-check green
```

**Expected, next ticks** (no PR review: no round, no approval, `review.angles = []`): the ticket goes **In Review** at the first tick that sees the PR ready, before its CI is green, then, once `required-check` is green, the bridge **merges the PR**: a squash pinned to the head SHA, as the App, the branch deleted; the ticket goes **To Deploy** (a release cycle is configured, so a merged ticket waits for a release; never Done here). One merge per tick.

Check:

```sh
gh pr view --repo "$REPO" "$BRANCH" --json state,mergedBy,mergeCommit --jq '{state, by: .mergedBy.login, sha: .mergeCommit.oid}'
show "$ID"                                          # To Deploy
spl-asdlc tick --file "$CFG" --dry-run              # on the host: nothing left to do for this PR
```

The tick log holds the merge decision and the transition, each once.

## (b) A release cycle

### 4. Open the cycle

To see the version and the notes first, in the clone, up to date with `main`, with the staging binary: `spl-asdlc release prepare --repository "$REPO" --dry-run --notes ../notes.md` (prints `version=`, `tag=` and `previous-tag=`, writes only the notes). Then:

```sh
gh workflow run release-cut.yml --repo "$REPO" --ref main -f action=open
# or, on the host: spl-asdlc release cut --file "$CFG" --product "$P" --open
gh run watch --repo "$REPO" "$(gh run list --repo "$REPO" --workflow release-cut.yml --limit 1 --json databaseId --jq '.[0].databaseId')"
```

**Expected on GitHub, with no gesture**:

1. `release-cut` (open) downloads the `staging` binary of spl-asdlc, checks it against the release's `SHA256SUMS`, creates `release/<YYYY-MM-DD-HH-MM>-utc` from `main` and runs `spl-asdlc release prepare` on it: the version computed from the commits since the last release `v<X.Y.Z>` (a `feat` among them bumps the minor, before 1.0), written into `Cargo.toml` and `Cargo.lock`, and the version's section added at the top of `CHANGELOG.md` (release-please's format: the heading with its compare link, one section per type, each entry with its PR and commit links). It commits `chore(release): v<version>` and pushes the branch with the first candidate `v<version>-rc.1` on that commit, in one push. No PR, no other workflow: the run's summary gives the version, the tag and the release class.
2. The tag starts `release-gate`: the build, then the suite `test` on the exact candidate, the prepared `CHANGELOG.md` in the evidence pack. Green. `release-gate-auto-rerun` runs after it and reruns nothing.

```sh
git fetch origin --tags
B=$(git ls-remote --heads origin 'release/*' | sed 's#.*refs/heads/##' | tail -n 1)
git log -1 --format='%s' "origin/$B"                     # chore(release): v<version>
git ls-remote --tags origin 'v*-rc.*'                    # v<version>-rc.1 on that commit
git show "origin/$B:Cargo.toml" | grep '^version'        # the new version
git show "origin/$B:CHANGELOG.md" | awk '/^## \[/ {n++} n == 1'   # its section on top, the ticket's PR listed
gh run list --repo "$REPO" --workflow release-gate.yml --limit 1
```

**Expected from the bridge, next ticks**: the **release issue** (its title names `spl-asdlc-sandbox` and the version), the candidate record, the label `release/spl-asdlc-sandbox-v<version>` on the release issue and on the embarked ticket `$ID`; once the gate is green, the **brief** on the release issue (the embarked ticket with its acceptance criteria, no business rule touched, the link to the gate's evidence, and that no QA angle is configured). No angle sub-item, no consolidation item.

```sh
spl-asdlc release status --file "$CFG" --product "$P"   # on the host: the GO and its inputs; exit 1 while there is no GO
show "$ID"                                              # To Deploy, with the release label
```

### 5. GO: the bridge tags and publishes the release

**Expected, next tick** once the gate is green: `release status` reads GO (gate green, no blocker, no QA angle, the candidate the release branch's tip), and the bridge, as the App, **creates the annotated tag `v<version>`** on the candidate's own commit (`Release v<version>`, naming `v<version>-rc.1`), then **publishes the GitHub release** `v<version>`, titled `spl-asdlc-sandbox <version>`, its notes the version's CHANGELOG section read at the candidate. A second tick tags and publishes nothing more.

Then, on GitHub, with no gesture, the release starts `release-publish`: **verify** (the candidate's gate green, the release tag on `v<version>-rc.1`'s commit, the release published by the App: `go-published-by-kind` in its summary), **publish** (prints `spl-asdlc-sandbox <version>, certified by v<version>-rc.1 at <sha>`), **close** (opens the merge-back PR from the release branch into `main`, waits for its `required-check`, merges it as a merge commit through the App, deletes the release branch).

```sh
git fetch origin --tags
test "$(git rev-parse "v<version>^{commit}")" = "$(git rev-parse "v<version>-rc.1^{commit}")" && echo "the release is the candidate"
gh release view --repo "$REPO" "v<version>" --json name,author,body --jq '{name, by: .author.login}'
gh run list --repo "$REPO" --workflow release-publish.yml --limit 1
gh pr list --repo "$REPO" --base main --state merged --limit 1     # the merge-back, a merge commit
git ls-remote --heads origin 'release/*'                           # empty
```

### 6. Closure

**Expected, next tick** after the release is published and the merge back merged: the **closure**. The embarked ticket `$ID`, To Deploy with every box ticked, goes **Done**; the release note is posted; the release issue is closed with its summary, which says who published the release (the App). Then the **archiving**: the ticket Done, then the release issue last, both archived (searchable and restorable in Linear).

```sh
show "$ID"                                              # Done, archivedAt set
spl-asdlc release status --file "$CFG" --product "$P"   # the cycle is over
spl-asdlc tick --file "$CFG" --dry-run                  # nothing to do
tail -n 30 "$LOG" | jq -c '{when, object, action, outcome}'
```

A second tick writes nothing (every write is idempotent): the tick log gets no new line.

## (c) A PR merged to main during a cycle is in the next release's notes

The cut prepares the notes along the git graph, never by date: a PR merged into `main` while a cycle is open is not in that cycle's release, and is in the next one's notes. Part (c) rides on (b)'s cycle: step 7 happens while it is open, step 8 after its closure.

### 7. While the cycle is open, merge a second ticket into main

Between step 4 and step 5 (the release branch exists, its release is not published), run steps 1 to 3 of part (a) again with a second ticket of the same shape: another small function and its unit tests (for instance `wave(name)` returning `Hi, <name>!`), a `feat` title naming the ticket. Those steps reuse `ID` and `BRANCH`, and step 2 starts from the existing clone (`git switch main && git pull` instead of the clone), so keep the first ticket aside:

```sh
ID1=$ID
# steps 1 to 3 with the second ticket, until the bridge has merged its PR
ID2=$ID
PR2=$(gh pr view --repo "$REPO" "$BRANCH" --json number --jq .number)
ID=$ID1                                                 # steps 5 and 6 follow the first ticket
```

**Expected**: the bridge merges the PR into `main` as in step 3, while the cycle runs on; the ticket `$ID2` goes **To Deploy**. It is not embarked in the open cycle: the release branch was cut before it, so its commit is not in the candidate's range, and the release issue and the brief do not list it. The open cycle goes on through steps 5 and 6 untouched; at its closure, `$ID2` stays To Deploy (its PR is not in that release's range).

```sh
git fetch origin
git merge-base --is-ancestor "$(gh pr view --repo "$REPO" "$PR2" --json mergeCommit --jq .mergeCommit.oid)" "origin/$B" || echo "not in the open cycle"
show "$ID2"                                             # To Deploy, without the open release's label
```

### 8. The next cycle lists it

After step 6 (the release `v<version>` published, the merge back merged), open the next cycle as in step 4:

```sh
gh workflow run release-cut.yml --repo "$REPO" --ref main -f action=open
```

**Expected**: the cut prepares `v<next>` from the commits since `v<version>`, which hold `$PR2`'s squash commit (the merge back brought `v<version>`'s commits into `main`, and the range leaves them out). The new section of `CHANGELOG.md` lists `$PR2` under **Features**, with its PR link, and nothing that `v<version>` already shipped. The bridge creates the release issue and labels `$ID2` with `release/spl-asdlc-sandbox-v<next>`; the brief lists it.

```sh
git fetch origin --tags
B2=$(git ls-remote --heads origin 'release/*' | sed 's#.*refs/heads/##' | tail -n 1)
git show "origin/$B2:CHANGELOG.md" | awk '/^## \[/ {n++} n == 1' | grep "#$PR2"   # the PR, in the new section
show "$ID2"                                                                        # To Deploy, with the new release's label
```

Then finish this cycle as steps 5 and 6: `$ID2` goes **Done** at its closure.

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
