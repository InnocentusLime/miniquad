# Contributing

I feel enthusiastic about maintaining my fork of miniquad and bringing it closer
to some sort of complete state. Please understand: I am alone 

These. guidelines are bare minimum.

## How To Write An Issue

1. Please use the appropriate issue template
2. This fork accepts only the following types of issues
    a. Bug reports
    b. Feature requests (when you want to add something that isn't there)
    c. If an issue is neither of the above, it will be rejected
2. If your issue is a bug report, please make sure it is reproducible and the instructions are clear
    a. Non-reproducible bug reports will be rejected
    b. Bug reports without any steps for reproduction will be rejected
    c. A simple piece of code for reproducing the bug will be always appreciated
3. If you issue is a feature request, please make sure the request is described clearly
    a. Additional rendering backends are not planned and will be rejected
    b. Features not possible to support on all miniquad platforms will be rejected
    c. Features requiring to boost minimal required OpenGL version will be rejected

## How To Make a PR

1. First of all, PRs are always welcome!
2. This fork accepts only the following types of PRs
    a. Bug fixes
    b. Feature additions
    c. API change
    d. If a PR is neither of the above, it will be rejected
3. PRs that redesign API may be merged, but are not guaranteed to
    a. You may updated API through a PR, but NOT through an issue
4. PRs that are known to lead to regressions will be rejected
    a. If a PR pulls in a large crate -- this is a regression
    b. If a PR significantly affects build times -- this is a regression
    c. If a PR breaks examples -- this is a regression
    d. Please, put best effort into making sure you did not introduce a regression

## Commits

Bad things may happen. Like stuff suddenly breaking in master. When this
happens, looking through the history of changes helps a lot. This is why
to save everyone's sanity the following commit writing rules exist:

1. Please keep your changes logically split among several commits
2. Please do not include repeated commits of your trying something like:
    1. commit1: `add thing`
    2. commit2: `fix add thing`
    3. commit3: `fix the fix`
3. Please keep the commit messages short and to the point
4. Please annotate the changed part of the codebase accordingly. Examples:
    1. `graphics: change the texture format`
    2. `linux-x11: change keyboard handling`
5. If your commits do not match the requirements, consider using `git rebase -i`