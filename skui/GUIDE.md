# SKUI DSL Syntax Guide

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
- But parantheses are required.

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
    - 오직 Root 컴퍼넌트내의 자식 컴퍼넌트에게만 부여될 수 있습니다
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
이 속성들은 주로 모든 컴퍼넌트들의 공통된 특성을 다룹니다. 하지만 모두가 반드시 공통이지는 않습니다.

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
Flex() { ... }  

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
    Relative(Vec<ValueKey>),
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

### 4. Rules
#### 4.1 Id 가 없는 컴퍼넌트는 Root 컨퍼넌트로 간주하며 하나만 존재하여야 합니다.
#### 4.2 Id 는 Root 컴퍼넌트의 자식들에게만 부여할 수 있습니다. 커스텀 컴퍼넌트에 붙일 수 없습니다.
#### 4.3 컴퍼넌트의 파라미터는 배열과 맵을 혼합할 수 없습니다.

## Notes
- Component blocks `{ }` and map values `{ }` are **context-sensitive** and distinguished by their position in the syntax.
- The DSL favors **readability and predictable parsing** over full CSS compatibility.
- 컴퍼넌트 파라미터와 프로퍼티는 무엇이 다릅니까? 파라미터는 컴퍼넌트들의 특성을 다루며 프로퍼티는 컴퍼넌트의 공통된 특성을 다룹니다.

---

## Error examples
- Case.1
```
CustomWidget() #hello .white_back {
	Flex( ${0}, ${1} ) {
		item( 0, Label( ${key} ) )
		item( 1, Button("OK") )
	}
}

Flex {
	CustomWidget( Row, Start, key="Hello" )
	Button("OK")
}
```
- 여기에는 2가지 문제가 존재합니다. 
  - 먼저 커스텀 컴퍼넌트는 아이디를 가질 수 없습니다.
  - Customized component can't have ID
    - Fix
      ```
      CustomWidget() .white_back {
      ...
    
      Flex {
          CustomWidget( Row, Start, "Hello" ) #hello
          Button("OK")
      }
      ```
  - 파라미터는 배열과 키방식을 혼합할 수 없습니다
    - {0} and {1} is index parameter but {key} is map parameter.
    - Fix 
      ```
      item( 0, Label( ${2} ) )
      ```
