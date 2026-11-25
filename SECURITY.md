# Security Policy

## Supported versions

plev is before its first release. Security fixes land on `main`.

## Reporting a vulnerability

Please do not open a public issue for a vulnerability.

Report it privately through
[GitHub's private vulnerability reporting](https://github.com/brennercruvinel/plev/security/advisories/new),
or write to **brenner@hoffresearch.com**. Include what is affected, how to
reproduce it and the impact you see.

You will get an answer saying the report was received, and then what was
found and what will be done. Once a fix is on `main`, the advisory is
published with credit to you, unless you prefer otherwise.

## What is in scope

- Decoders of untrusted input: the `.monster` codec, the importers (Lottie,
  SVG, React, HTML, Figma, gpui) and recorded sessions.
- The agent port: a local socket through which an agent observes and
  operates an app. Anything that lets an agent act outside the app's
  dispatcher, or on state it was not shown, is a vulnerability.
- Memory safety in `unsafe` code and at the platform boundaries.

A dependency with a known advisory can be reported in a regular issue with
the `security` label.
