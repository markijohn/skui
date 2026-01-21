# skui
Simple and simple

## WARNING
* VERY EARLY DEVELOPMENT STAGES
* It may be better to create a separate project to use this crate rather than using it directly.

## crates
* [`skui`](https://github.com/markijohn/skui/tree/main/crates/skui) : parse dsl
* [`skui_masonry_macro`](https://github.com/markijohn/skui/tree/main/skui_masonry_macro) : generate masonry code from parsed dsl
* [`skui_egui_macro`](https://github.com/markijohn/skui/tree/main/skui_egui_macro) : generate egui code from parsed dsl
* [`skui_bevy_macro`](https://github.com/markijohn/skui/tree/main/skui_egui_macro) : generate bevy ui code from parsed dsl
* [`skui_masonry_example`](https://github.com/markijohn/skui/tree/main/skui_masonry_example) : preview and runtime generation
* [`skui_egui_example`](https://github.com/markijohn/skui/tree/main/skui_masonry_example) : preview and runtime generation
* [`skui_bevy_example`](https://github.com/markijohn/skui/tree/main/skui_masonry_example) : preview and runtime generation

## Screenshot

## Try it online
* [ ] todo

## Quick Example
* Cargo.toml
```toml
[dependencies]
druid = {git="https://github.com/linebender/druid.git"}
druid-xml-macro = {git="https://github.com/markijohn/druid-xml.git"}
```

## I HATE MAGIC CODE (skui_masonry_macro)
```
Flex { background-color: black; padding:1px }
#list { border: 1px solid yellow }
.myBtn { border: 2px }
#myFlex { border:2px }
.background_white { background-color: WHITE }


Flex(MainFill) #myFlex .background_white {
	myProperty1 : "data"
	propertyMap : {key=1, key2=true}
	propertyAnother : [ 1,2,3 ]
	FlexItem(1.0) {  Button("FlexItem")  }
	FlexItem(2.0, Button("FlexItem2"))
	Button()
}

Grid(2,3) {
	Label()
}
```

```rust
todo!()
```

## I HATE MAGIC CODE (skui_egui_macro)

* Rust code
```
```
```rust
todo!()
```


## Style
<table>
 <thead>
   <td>ATTRIBUTE</td>
   <td>VALUE(example)</td>
   <td>AVAILABLE WIDGET</td>
   <td>DESCRIPTION</td>
 </thead>
 <tbody>
 <tr>
   <td>border</td>
   <td>1 solid black<br/>1px solid yellow</td>
   <td>all</td>
   <td>Only solid brush type is supported</td>
 </tr>
 <tr>
   <td>padding</td>
   <td>5<br/>10 5<br/>10 15 15 10</td>
   <td>all</td>
   <td>(top,right,bottom,left)<br/>(top,bottom) (left,right)<br/>(top) (right) (bottom) (left)</td>
 </tr>
 <tr>
   <td>background-color</td>
   <td>rgb(0,255,255)<br/>rgba(0,255,255,88)<br/>#96ab05</td>
   <td>all</td>
 </tr>
 <tr>
   <td>color</td>
   <td>rgb(0,255,255)<br/>rgba(0,255,255,88)<br/>#96ab05</td>
   <td>label, button</td>
   <td>text color</td>
 </tr>
 <tr>
   <td>width</td>
   <td>25<br/>25px</td>
   <td>all</td>
   <td>percentage size not yet support(or impossible)</td>
 </tr>
 <tr>
   <td>height</td>
   <td>25<br/>25px</td>
   <td>all</td>
   <td>percentage size not yet support(or impossible)</td>
 </tr>
 <tr>
  <td>transition</td>
   <td>2s background-color linear<br/><br/>font-size<br/>margin<br/>padding<br/>color<br/>border</td>
   <td>all</td>
   <td>for hover, focus, active animation</td>
 </tr>
 </tbody>
</table>

## Widget

<table>
 <thead>
   <td>LOCALNAME</td>
   <td>ATTRIBUTES</td>
 </thead>
 <tbody>
 <tr>
   <td>flex</td>
   <td>must_fill_main_axis<br/>
   flex<br/>
   axis_alignment<br/>
   cross_axis_alignment<br/></td>
 </tr>
 <tr>
   <td>label</td>
   <td>flex<br/>line-break</td>
 </tr>
 <tr>
   <td>button</td>
   <td>flex</td>
 </tr>
 <tr>
   <td>checkbox</td>
   <td>flex</td>
 </tr>
 <tr>
   <td>textbox</td>
   <td>flex</td>
 </tr>
 <tr>
   <td>image(not yet)</td>
   <td>flex</td>
 </tr>
 <tr>
   <td>list(not yet)</td>
   <td>flex</td>
 </tr>
 <tr>
   <td>scroll(not yet)</td>
   <td>flex</td>
 </tr>
 <tr>
   <td>slider</td>
   <td>flex<br/>min<br/>max<br/></td>
 </tr>
 <tr>
   <td>spinner(some proble on wasm)</td>
   <td>flex</td>
 </tr>
 <tr>
   <td>split</td>
   <td>flex<br/>split_point<br/>min_size<br/>bar_size<br/>min_bar_area<br/>draggable<br/>solid_bar</td>
 </tr>
 <tr>
   <td>stepper</td>
   <td>flex<br/>min<br/>max<br/>step<br/>wraparound</td>
 </tr>
 <tr>
   <td>switch</td>
   <td>flex<br/></td>
 </tr>
 <tr>
   <td>painter(not yet)</td>
   <td>flex<br/></td>
 </tr>
 <tr>
   <td>container</td>
   <td>flex<br/></td>
 </tr>
 </tbody>
</table>

## TODO
* Load xml from project path
* Animation : CSS `transition` and `Animation`
* Drawable widget : like [`Android Drawable`](https://developer.android.com/guide/topics/resources/drawable-resource)