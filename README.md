# Yarte [![Documentation](https://docs.rs/yarte/badge.svg)](https://docs.rs/yarte/) [![Latest version](https://img.shields.io/crates/v/yarte.svg)](https://crates.io/crates/yarte) [![codecov](https://codecov.io/gh/rust-iendo/yarte/branch/master/graph/badge.svg)](https://codecov.io/gh/rust-iendo/yarte) [![Build status](https://api.travis-ci.org/rust-iendo/yarte.svg?branch=master)](https://travis-ci.org/rust-iendo/yarte) [![Windows build](https://ci.appveyor.com/api/projects/status/github/rust-iendo/yarte?svg=true)](https://ci.appveyor.com/project/botika/v-htmlescape) [![Downloads](https://img.shields.io/crates/d/yarte.svg)](https://crates.io/crates/yarte)
Yarte stands for **Y**et **A**nother **R**ust **T**emplate **E**ngine, and is a handlebars-like template engine implemented in rust. 
Developers familiar with Handlebars will find syntax is very intuitive. Create templates invoking structs, using conditionals, loops, rust code, and predefined functions and templates! Yarte is optimized using ... to make your templates as fast as possible (literally).

## Roadmap

## Setup

## Getting started

In order to use a struct in the template  you will have to call the procedural macro `Template`. For example, in the following code we are going to use struct `VariablesTemplate`, to then define `s` as a `VariablesTemplate` with content.

    #[derive(Template)]
    #[template(path = "hello.html")]
    struct VariablesTemplate<'a> {
        title: &'a str,
        body: &'a str,
    }
    
    let s = VariablesTemplate {
            title: "My Title",
            body: "My Body",
        };
    
Now that our struct is defined lets use it in a HTML template. Yarte templates look like regular HTML, with embedded yarte expressions.

Let's say file `hello.html` looks like this:

    <div class="entry">
      <h1>{{title}}</h1>
      <div class="body">
        {{body}}
      </div>
    </div>

When rendered, the following html code will be created, as expected:

    <div class="entry">
      <h1> My Title </h1>
      <div class="body">
        My Body
      </div>
    </div>


## Paths
Yarte supports paths

          


## Comments

## HTML
Yarte HTML-escapes values returned by a {{expression}}. If you don't want Yarte to escape a value, use the "triple-stash", {{{. For example having the following struct:

    {
      title: "All about <p> Tags",
      body: "<p>This is a post about &lt;p&gt; tags</p>"
    }
 and the following template:

    <div class="entry">
      <h1>{{title}}</h1>
      <div class="body">
        {{{body}}}
      </div>
    </div>

will result in:
    
    <div class="entry">
      <h1>All About &lt;p&gt; Tags</h1>
      <div class="body">
        <p>This is a post about &lt;p&gt; tags</p>
      </div>
    </div>

## Helpers

### Built-in
#### If, else, and else if helper
    {{#if isLiked}}
      Liked!
    {{else if isSeen}}
      Seen!
    {{else}}
      Sorry ...
    {{\if}}
    
#### With helper
    {{#with author}}
      <p>{{name}}</p>
    {{else}}
      <p class="empty">No content</p>
    {{/with}}
    
#### Each helper
    {{#each }} {{\each}}
    
## Literal
Booleans, integers, floating points, ... are not escaped for better performance. It is recomended to use these types if possible.

## Partials
Partials can be used to generate faster code using a pre defined functions.

    {{> partial }}

## Rust code
Yarte provides you with the possibility to use raw rust code within the HTML files. This is limited, but most of esential syntax is soppurted.
    
    {{ let a = 1 }}
    {{ let Some(user) = getUserFromDB() }}
