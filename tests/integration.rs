use oxidoc_highlight::highlight;

// ── Rust ─────────────────────────────────────────────────────────────

#[test]
fn rust_full_example() {
    let code = r#"use std::collections::HashMap;

/// A doc comment
fn main() {
    let mut map: HashMap<String, i32> = HashMap::new();
    map.insert("hello".to_string(), 42);
    println!("value: {}", map.get("hello").unwrap());
}

#[derive(Debug, Clone)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}
"#;
    let out = highlight(code, "rust");
    assert!(out.contains("tok-keyword")); // use, fn, let, mut, struct, impl
    assert!(out.contains("tok-type")); // HashMap, String, i32, Point, f64
    assert!(out.contains("tok-string")); // "hello"
    assert!(out.contains("tok-comment")); // /// A doc comment
    assert!(out.contains("tok-builtin")); // println!
    assert!(out.contains("tok-attr")); // #[derive(Debug, Clone)]
    assert!(out.contains("tok-number")); // 42
    assert!(out.contains("tok-function")); // main, insert, distance
    assert!(out.contains("tok-variable")); // self
}

#[test]
fn rust_raw_string_complex() {
    let code = r###"let s = r##"hello "world" here"##;"###;
    let out = highlight(code, "rust");
    assert!(out.contains("tok-string"));
}

#[test]
fn rust_lifetime_vs_char() {
    let out1 = highlight("'a", "rust");
    assert!(out1.contains("tok-variable")); // lifetime

    let out2 = highlight("'x'", "rust");
    assert!(out2.contains("tok-string")); // char literal
}

// ── JavaScript ───────────────────────────────────────────────────────

#[test]
fn js_full_example() {
    let code = r#"import { useState } from 'react';

const App = () => {
  const [count, setCount] = useState(0);
  const obj = { name: "test", value: 42 };

  // Handle click
  function handleClick() {
    setCount(count + 1);
    console.log(`Count: ${count}`);
  }

  return null;
};

/* Block comment */
const regex = /pattern/gi;
"#;
    let out = highlight(code, "javascript");
    assert!(out.contains("tok-keyword")); // import, const, function, return
    assert!(out.contains("tok-string")); // strings, template literal, regex
    assert!(out.contains("tok-comment")); // both comment styles
    assert!(out.contains("tok-number")); // 0, 42, 1
    assert!(out.contains("tok-function")); // handleClick, useState
    assert!(out.contains("tok-builtin")); // console
}

#[test]
fn js_regex_vs_division() {
    let out = highlight("const x = a / b / c;", "js");
    assert!(out.contains("tok-operator")); // division

    let out = highlight("const r = /test/g;", "js");
    assert!(out.contains("tok-string")); // regex
}

#[test]
fn js_template_literal_nested() {
    let out = highlight("`hello ${a + b} world`", "js");
    assert!(out.contains("tok-string"));
}

// ── TypeScript ───────────────────────────────────────────────────────

#[test]
fn ts_types_and_interfaces() {
    let code = r#"interface User {
  name: string;
  age: number;
}

type Result<T> = T | null;

@Component
class MyComponent {
  private value: number = 0;
}"#;
    let out = highlight(code, "typescript");
    assert!(out.contains("tok-keyword")); // interface, class, private
    assert!(out.contains("tok-type")); // string, number, User (PascalCase)
    assert!(out.contains("tok-attr")); // @Component
}

// ── Python ───────────────────────────────────────────────────────────

#[test]
fn python_full_example() {
    let code = r#"from typing import Optional, List

@dataclass
class Point:
    """A 2D point."""
    x: float
    y: float

    def distance(self, other: 'Point') -> float:
        return ((self.x - other.x) ** 2 + (self.y - other.y) ** 2) ** 0.5

# Create points
points: List[Point] = [Point(0, 0), Point(3, 4)]
print(f"Distance: {points[0].distance(points[1])}")
"#;
    let out = highlight(code, "python");
    assert!(out.contains("tok-keyword")); // from, import, class, def, return
    assert!(out.contains("tok-attr")); // @dataclass
    assert!(out.contains("tok-string")); // docstring, f-string
    assert!(out.contains("tok-comment")); // # Create points
    assert!(out.contains("tok-variable")); // self
    assert!(out.contains("tok-builtin")); // print
    assert!(out.contains("tok-function")); // distance
}

// ── Bash ─────────────────────────────────────────────────────────────

#[test]
fn bash_full_example() {
    let code = r#"#!/bin/bash

# Deploy script
export APP_NAME="myapp"

if [ -f "$HOME/.config" ]; then
    echo "Config found"
    source "$HOME/.config"
fi

for file in *.txt; do
    echo "Processing: $file"
    cat "$file" | grep -i "pattern"
done

result=$(curl -s https://api.example.com)
echo "${result}"
"#;
    let out = highlight(code, "bash");
    assert!(out.contains("tok-comment")); // # comments
    assert!(out.contains("tok-keyword")); // if, then, fi, for, do, done
    assert!(out.contains("tok-string")); // double-quoted strings
    assert!(out.contains("tok-variable")); // $HOME, $file, ${result}
    assert!(out.contains("tok-builtin")); // echo
}

// ── HTML ─────────────────────────────────────────────────────────────

#[test]
fn html_full() {
    let code = r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <!-- Title -->
    <title>Test</title>
  </head>
  <body class="main">
    <h1 id="title">Hello</h1>
  </body>
</html>"#;
    let out = highlight(code, "html");
    assert!(out.contains("tok-keyword")); // tags
    assert!(out.contains("tok-attr")); // attributes
    assert!(out.contains("tok-string")); // attribute values
    assert!(out.contains("tok-comment")); // <!-- Title -->
}

// ── JSON ─────────────────────────────────────────────────────────────

#[test]
fn json_full() {
    let code = r#"{
  "name": "oxidoc",
  "version": "0.1.0",
  "count": 42,
  "enabled": true,
  "data": null,
  "items": [1, 2, 3]
}"#;
    let out = highlight(code, "json");
    assert!(out.contains("tok-property")); // keys
    assert!(out.contains("tok-string")); // values
    assert!(out.contains("tok-number")); // 42, 1, 2, 3
    assert!(out.contains("tok-keyword")); // true, null
}

// ── TOML ─────────────────────────────────────────────────────────────

#[test]
fn toml_full() {
    let code = r#"[package]
name = "oxidoc"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1.0", features = ["derive"] }

# Dev only
[dev-dependencies]
criterion = "0.5"
"#;
    let out = highlight(code, "toml");
    assert!(out.contains("tok-keyword")); // section headers
    assert!(out.contains("tok-property")); // keys
    assert!(out.contains("tok-string")); // values
    assert!(out.contains("tok-comment")); // # Dev only
}

// ── YAML ─────────────────────────────────────────────────────────────

#[test]
fn yaml_full() {
    let code = r#"---
apiVersion: v1
kind: Deployment
metadata:
  name: "my-app"
  labels:
    app: web
spec:
  replicas: 3
  enabled: true
  # Comment
"#;
    let out = highlight(code, "yaml");
    assert!(out.contains("tok-keyword")); // ---
    assert!(out.contains("tok-property")); // keys
    assert!(out.contains("tok-string")); // "my-app"
    assert!(out.contains("tok-comment")); // # Comment
}

// ── Diff ─────────────────────────────────────────────────────────────

#[test]
fn diff_full() {
    let code = r#"diff --git a/file.rs b/file.rs
--- a/file.rs
+++ b/file.rs
@@ -1,3 +1,4 @@
 context
-removed
+added
+also added
"#;
    let out = highlight(code, "diff");
    assert!(out.contains("tok-keyword")); // diff, ---, +++
    assert!(out.contains("tok-attr")); // @@ header
    assert!(out.contains("tok-string")); // + lines
    assert!(out.contains("tok-comment")); // - lines
}

// ── JSX ──────────────────────────────────────────────────────────────

#[test]
fn jsx_react_component() {
    let code = r#"function App() {
  const [count, setCount] = useState(0);
  return (
    <div className="app">
      <Header title="Hello" />
      <Button onClick={() => setCount(count + 1)} disabled={false}>
        Count: {count}
      </Button>
      <>{items.map(item => <Item key={item.id} />)}</>
    </div>
  );
}"#;
    let out = highlight(code, "jsx");
    assert!(out.contains("tok-keyword")); // function, return, const + <Header, <Button
    assert!(out.contains("tok-attr")); // className, onClick, title, disabled, <div, <Item
    assert!(out.contains("tok-string")); // "app", "Hello"
    assert!(out.contains("tok-function")); // App, useState, setCount
}

// ── TSX ──────────────────────────────────────────────────────────────

#[test]
fn tsx_typed_component() {
    let code = r#"interface Props {
  name: string;
  count: number;
}

const Counter: React.FC<Props> = ({ name, count }) => (
  <div>
    <h1>{name}</h1>
    <span className="count">{count}</span>
  </div>
);
"#;
    let out = highlight(code, "tsx");
    assert!(out.contains("tok-keyword")); // interface, const
    assert!(out.contains("tok-type")); // string, number, Props, React
}

// ── PHP ──────────────────────────────────────────────────────────────

#[test]
fn php_full_example() {
    let code = r#"<?php

namespace App\Controllers;

use App\Models\User;

#[Route('/users')]
class UserController extends Controller
{
    private readonly UserRepository $repo;

    public function __construct(UserRepository $repo)
    {
        $this->repo = $repo;
    }

    /**
     * Get all users
     */
    public function index(): array
    {
        $users = User::all();
        $names = array_map(fn($u) => $u->name, $users);
        return ['users' => $names, 'count' => count($users)];
    }

    public function show(int $id): ?User
    {
        $user = $this->repo->find($id);
        if ($user === null) {
            throw new NotFoundException("User $id not found");
        }
        return $user;
    }
}
"#;
    let out = highlight(code, "php");
    assert!(out.contains("tok-keyword")); // <?php, class, function, public, return, etc.
    assert!(out.contains("tok-variable")); // $repo, $this, $users, $id
    assert!(out.contains("tok-type")); // UserController, Controller, User, UserRepository
    assert!(out.contains("tok-function")); // index, show, __construct, find
    assert!(out.contains("tok-builtin")); // array_map, count
    assert!(out.contains("tok-string")); // strings
    assert!(out.contains("tok-comment")); // docblock
    assert!(out.contains("tok-attr")); // #[Route('/users')]
    assert!(out.contains("tok-operator")); // =>, ->, ::
    assert!(out.contains("tok-property")); // name, repo (after ->)
}

// ── Edge Cases ───────────────────────────────────────────────────────

#[test]
fn all_languages_no_panic_on_empty() {
    let langs = oxidoc_highlight::supported_languages();
    for lang in langs {
        let _ = highlight("", lang);
    }
}

#[test]
fn all_languages_no_panic_on_garbage() {
    let garbage = "}{][)(///**/\"\"'''```\n\r\t\x00🎉中文العربية";
    let langs = oxidoc_highlight::supported_languages();
    for lang in langs {
        let _ = highlight(garbage, lang);
    }
}

#[test]
fn very_long_line_no_hang() {
    let long = "x".repeat(100_000);
    let _ = highlight(&long, "rust");
}
