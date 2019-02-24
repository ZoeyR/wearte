# \[WIP] wearte [![Documentation](https://docs.rs/wearte/badge.svg)](https://docs.rs/wearte/) [![Latest version](https://img.shields.io/crates/v/wearte.svg)](https://crates.io/crates/wearte) [![Build status](https://api.travis-ci.org/rust-iendo/wearte.svg?branch=master)](https://travis-ci.org/rust-iendo/wearte) [![Windows build](https://ci.appveyor.com/api/projects/status/github/rust-iendo/wearte?svg=true)](https://ci.appveyor.com/project/botika/v-htmlescape) [![Downloads](https://img.shields.io/crates/d/wearte.svg)](https://crates.io/crates/wearte)
wearte stands for **W**ow **E**ven **A**nother **R**ust **T**emplate **E**ngine, it is one of the fastest rust template engines. It uses a Handlebars-like syntax.

This crate was forked from [yarte](https://github.com/rust-iendo/yarte) with fixes for the snarky licensing issues. yarte itself is a direct descendant of [askama](https://github.com/djc/askama). You can find copies of their licenses in LICENSE-MIT.

## Why a derive template engine?
There are many templates engines based on mustache or/and handlebars,
I have not known any that derives the compilation of templates to the compiler (like [askama](https://github.com/djc/askama)).
By deriving this task from another process, we can optimize the instructions 
generated with our own tools or those of third parties such as LLVM. 
This is impossible in other cases creating a bottleneck in our web servers 
that reaches milliseconds. Because of this, `wearte` puts the template in priority 
by allocating its needs statically. Thus, we write faster than the macro `write!`, 
easy parallelism and with simd in its default html escape. 

In conclusion a derive is used to be the fastest and simplest.

## Getting started
Add wearte dependency to your Cargo.toml file:

```toml
[dependencies]
wearte = "0.0.1"
```

In order to use a struct in the template  you will have to call 
the procedural macro `Template`. For example, in the following 
code we are going to use struct `CardTemplate`, to then 
define `s` as a `CardTemplate` with content.

```rust
use wearte::Template;

#[derive(Template)]
#[template(path = "hello.html")]
struct CardTemplate<'a> {
    title: &'a str,
    body: &'a str,
}

let template = CardTemplate {
    title: "My Title",
    body: "My Body",
};
```
    
Now that our struct is defined lets use it in a template. 
wearte templates look like regular text, with embedded wearte expressions.

Let's say file `hello.html` looks like this:
```handlebars
<div class="entry">
  <h1>{{title}}</h1>
  <div class="body">
    {{body}}
  </div>
</div>
```

And call your template for allocate the result in `String` and return 
it wrapped with wearte::Result
```rust
template.call()
```

```html
<div class="entry">
  <h1> My Title </h1>
  <div class="body">
    My Body
  </div>
</div>
```

## Templating
wearte uses opening characters `{{` and closing 
characters `}}` to parse the inside depending 
on the feature used. Most of the features are 
defined by Handlebars such as paths, comments, 
html, helpers and partials. Others such as 
adding rust code to a template, are obviously 
defined by wearte.    

```rust
// precompile your template
#[derive(Template)]
#[template(source = "Hello, {{ name }}!", ext = "txt")]
struct HelloTemplate<'a> {
    name: &'a str,
}

assert_eq!(
    "Hello, world!", 
    HelloTemplate { name: "world" }.call().unwrap() // then call it.
); 
```

## Comments

```handlebars
{{!   Comments can be written  }}
{{!--  in two different ways --}}
```

## HTML
wearte HTML-escapes values returned by a `{{expression}}`. 
If you don't want wearte to escape a value, use the 
"triple-stash", `{{{`. For example having the following 
struct:

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
wearte provides you with the possibility to use raw rust code within the HTML files. This is limited, but most of essential syntax is supported.
    
```handlebars
{{#with getUser(id)?-}}
    Hello, {{#if isAdmin || isDev }}Mr. {{\if}}{{ user }}
{{/-with}}
```

```handlebars
Hello, {{#each conditions}}
    {{#-if let Some(check) = cond }}
        {{#-if check }}
            {{ let cond = if check { "&foo" } else { "&"} }}
            {{
                if check {
                    cond
                } else if let Some(cond) = key.cond {
                    if cond {
                        "1"
                    } else {
                        "2"
                    }
                } else {
                   "for"
                }
            }}
        {{- else if let Some(_) = cond }}
        {{- else if let Some(cond) = key.check }}
            {{#-if cond -}}
                baa
            {{/-if }}
        {{- else -}}
            {{ cond.is_some() }}
        {{/-if-}}
        {{ cond.is_some() && true }}
    {{-else if let Some(cond) = check }}
        {{#-if cond -}}
            bar
        {{/-if}}
    {{- else -}}
        None
    {{/-if
}}{{/each}}!
```

```handlebars
{{ let mut a = name.chars() }}

{{  
    let b: String = loop {
        if a.next().is_none() && true {
            let mut a = name.repeat(1);
            a.push('!');
            break a.repeat(2);
        } else {
            continue;
        }
    }
}}

{{ b }}
```

```handlebars
{{ let doubled = a.iter().map(|x| x * 2).collect::<Vec<_>>() }}
{{ let doubled: Vec<usize> = a.iter().map(|x| x * 2).collect() }}

{{#each doubled -}}
    {{ key + 1 }}
{{/-each}}
```

## Roadmap
- [ ] Minimize html5 at literal
- [ ] Derive builders for generate defined helpers and filters
- [ ] `>|` filters on fmt::Formatter
- [ ] Concatenate filters, unix like, on fmt::Formatter (when is possible)
- [ ] ... you can open a issue!