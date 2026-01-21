# DSL Syntax Guide

This document describes a simple DSL for defining **UI components** and **styles**.
The language is designed to be easy to read, easy to parse, and familiar to users with CSS or UI frameworks.

---

## Overview

This DSL has two main concepts:

- **Components**: UI elements declared with a name and optional parameters
- **Styles**: CSS-like style definitions applied to components

---

## 1. Component Syntax

### What is a Component?

Any expression that starts with `Ident(...)` is recognized as a **Component**.

Ident(arg1, arg2, ...)

---

### 1.1 Component Name

- `Ident` represents the component type name.
- Examples:
    - `Flex`
    - `Button`
    - `Grid`

---

### 1.2 Constructor Arguments

- Values inside `()` are treated as constructor arguments.
- Arguments are separated by commas.
- Arguments are optional.

Examples:

Button()  
Flex(1.0, true)  
Grid(2, 3)

---

### 1.3 Style Selectors (`#`, `.`)

Style selectors can follow a component declaration.

Example:

Flex() #main .highlight

Rules:

- `#id`
    - Defines a **component ID**
    - Only **one** ID is allowed
- `.class`
    - Defines a **component class**
    - Multiple classes are allowed
- The order of `#id` and `.class` does not matter

---

### 1.4 Component Body `{ }`

If a component declaration is followed by `{ }`, it defines the **component body**.
Body is optional.

Example:
```
Flex() {
    padding: 10
    Button("OK")
}
```
Inside the component body, you can place:

#### Properties

Properties are defined using the `key: value` syntax.

Examples:

width: 100  
enabled: true  
title: "Hello" 
array: [1,2,3,4] \
map: {key:"Value", number:83215} 

#### Child Components

Child components can be nested directly inside the body.

Examples:

Button("OK")  
Label("Hello")

---

## 2. Style Syntax

### What is a Style?

If an `Ident`, `#Ident`, or `.ident` is followed directly by `{ }`, it is treated as a **style definition**, not a component.

Example:

Button {
    border: 1px solid black
}

---

### Style Examples
```css
#main {
    padding: 10
}

.primary {
    background-color: blue
}

Flex {
    gap: 8
}
```
Notes:

- Style syntax is **CSS-like**
- It does **not fully conform to CSS grammar**
- Styles are matched and applied to components based on selectors

---

## 3. Value Types

The following value types can be used in properties and arguments.

Rust representation:
```
pub enum Value {
    Ident(String),
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
    Closure(String),
    Component(Component),
}
```
---

### 3.1 Value Examples

enabled: true  
count: 10  
title: "Hello"  
items: [1, 2, 3]  
options: { key1: 1, key2: false }  
child: Button("OK")

---

### 3.2 Arrays

Arrays are defined using square brackets `[]`.

Examples:

values: [1, 2, 3]  
flags: [true, false, true]

---

### 3.3 Maps

Maps are defined using curly braces `{}` with key-value pairs.

Example:

options: { key1: 1, key2: false }

---

## Notes

- Component blocks `{ }` and map values `{ }` are **context-sensitive** and distinguished by their position in the syntax.
- The DSL favors **readability and predictable parsing** over full CSS compatibility.

---

## Design Goals
- Simple and simple