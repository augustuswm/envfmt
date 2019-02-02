workflow "Commit" {
  on = "push"
  resolves = ["test"]
}

action "test" {
  uses = "docker://clux/muslrust"
  runs = "cargo test"
}
