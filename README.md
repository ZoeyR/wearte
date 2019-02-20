# \[WIP] Yarte [![Documentation](https://docs.rs/yarte/badge.svg)](https://docs.rs/yarte/) [![Latest version](https://img.shields.io/crates/v/yarte.svg)](https://crates.io/crates/yarte) [![Build status](https://api.travis-ci.org/rust-iendo/yarte.svg?branch=master)](https://travis-ci.org/rust-iendo/yarte) [![Windows build](https://ci.appveyor.com/api/projects/status/github/rust-iendo/yarte?svg=true)](https://ci.appveyor.com/project/botika/v-htmlescape) [![Downloads](https://img.shields.io/crates/d/yarte.svg)](https://crates.io/crates/yarte)
Yarte stands for **Y**et **A**nother **R**ust **T**emplate **E**ngine, is the fastest template engine. Uses a Handlebars-like syntaxis, well known and intuitive. Yarte is an optimized, and easy-to-use rust crate, with which developers can create logic around their HTML templates using using conditionals, loops, rust code, and predefined functions and using templates within templates.

## Why a derive template engine?
There are many templates engines based on mustache or/and handlebars,
I have not known any that derives the compilation of templates to the compiler (like [askama](https://github.com/djc/askama)).
By deriving this task from another process, we can optimize the instructions 
generated with our own tools or those of third parties such as LLVM. 
This is impossible in other cases creating a bottleneck in our web servers 
that reaches milliseconds. Because of this, `yarte` puts the template in priority 
by allocating its needs statically. Thus, we write faster than the macro `write!`, 
easy parallelism and with simd in its default html escape. 

In conclusion a derive is used to be the fastest and simplest.

## Getting started
Add Yarte dependency to your Cargo.toml file:

```toml
[dependencies]
yarte = "0.0"
```

In order to use a struct in the template  you will have to call the procedural macro `Template`. For example, in the following code we are going to use struct `VariablesTemplate`, to then define `s` as a `VariablesTemplate` with content.

```rust
use yarte::Template;

#[derive(Template)]
#[template(path = "hello.html")]
struct VariablesTemplate<'a> {
    title: &'a str,
    body: &'a str,
}

let template = VariablesTemplate {
    title: "My Title",
    body: "My Body",
};
```
    
Now that our struct is defined lets use it in a HTML template. Yarte templates look like regular HTML, with embedded yarte expressions.

Let's say file `hello.html` looks like this:

```handlebars
<div class="entry">
  <h1>{{title}}</h1>
  <div class="body">
    {{body}}
  </div>
</div>
```

When rendered, 
```rust
template.to_string();
```

the following html characters will be created, as expected:

```html
<div class="entry">
  <h1> My Title </h1>
  <div class="body">
    My Body
  </div>
</div>
```

## Templating
Yarte uses opening characters `{{` and closing characters `}}` to parse the inside depending on the feature used. Most of the features are defined by Handlebars such as paths, comments, html, helpers and partials. Others such as adding rust code to a template, are obviously defined by Yarte.    

## Paths
Yarte uses Paths so that the template can refer to the values given when creating a new structure. For this simple example we are going to write our template using attributtes `source` and `ext`.

```rust
#[derive(Template)]
#[template(source = "Hello, {{ name }}!", ext = "txt")]
struct HelloTemplate<'a> {
    name: &'a str,
}
```

Above we can see the path `name`

```rust
let t = HelloTemplate { name: "world" }; // instantiate your struct
assert_eq!("Hello, world!", t.render().unwrap()); // then render it.
```

## Comments

```handlebars
{{!   Comments can be written  }}
{{!--  in two different ways --}}
```

## HTML
Yarte HTML-escapes values returned by a {{expression}}. If you don't want Yarte to escape a value, use the "triple-stash", {{{. For example having the following struct:

```rust
let t = CardTemplate {
  title: "All about <p> Tags",
  body: "<p>This is a post about &lt;p&gt; tags</p>"
};
```
and the following template:

```handlebars
<div class="entry">
  <h1>{{title}}</h1>
  <div class="body">
    {{{body}}}
  </div>
</div>
```

will result in:
    
```handlebars
<div class="entry">
  <h1>All About &lt;p&gt; Tags</h1>
  <div class="body">
    <p>This is a post about &lt;p&gt; tags</p>
  </div>
</div>
```

## Helpers

### Built-in
#### If, else, and else if helper
```handlebars
{{#if isLiked}}
  Liked!
{{else if isSeen}}
  Seen!
{{else}}
  Sorry ...
{{\if}}
```
   
#### With helper
```rust
let author = Author {
    name: "J. R. R. Tolkien"
};
```

```handlebars
{{#with author}}
  <p>{{name}}</p>
{{/with}}
```

#### Each helper
```handlebars
{{#each into_iter}} 
    {{#- if first || last -}}
        {{ index }} 
    {{- else -}}
        {{ index0 }} 
    {{/-if }} {{ key }} 
{{\-each}}
```

#### Unless helper
```handlebars
{{#unless isAdministrator-}} 
  Ask administrator.
{{\-unless}}
```
    
#### \[WIP] Log helper
```handlebars
{{#log }} {{log\}}
```
    
#### \[WIP] Lookup helper
```handlebars
{{#lookup }} {{\lookup}}
```

### \[WIP] User-defined
In order to create a user-defined helper ..

## Literal
Booleans, integers, floating points, ... are not escaped for better performance. It is recomended to use these types if possible.

## Partials
Partials can be used to generate faster code using a pre defined functions.

```handlebars
{{> path/to/file }}
```
## Rust code
Yarte provides you with the possibility to use raw rust code within the HTML files. This is limited, but most of esential syntax is soppurted.
    
```handlebars
{{ let user = getUser() }}

{{# if let Ok(user) = user }}
    Hello, {{ user }}
{{- else }}
    How are you?
{{\-if}}
```


## Support

[Patreon](https://www.patreon.com/r_iendo)
