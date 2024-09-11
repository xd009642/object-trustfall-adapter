# Using `trustfall_stubgen`

So first step:

```
cargo install trustfall_stubgen
```

Then I wrote a graphql schema for my types and the data to generate and made my
stubs with:

```
mkdir tmp
trustfall_stubgen --schema schema.graphql --target tmp/ 
```

After copying this into my source directory and activating the module it was time
to get to work.

## Intial Thoughts on Generated Code

First the schema:

```
schema {
    query: RootSchemaQuery 
}

type RootSchemaQuery {
    text_section: [DecodedInstruction!]!

    getInstruction(address: Int!): DecodedInstruction

    debug_info: [SourceLocation!]!
    
    getLocation(address: Int!): SourceLocation
    getFileLocations(file: String!): [SourceLocation]
    getFileInstructions(file: String!): [DecodedInstruction]

}

type SourceLocation {
    """
    The name of the source code file
    """
    file: String!
    """
    The start line of the location
    """
    line: Int!
    """
    The column used - or null if this is a leftmost column
    """
    column: Int
}

type DecodedInstruction {
    """
    Address in memory of the instruction (this is the same as the Instruction Pointer)
    """
    address: Int!
    """
    Name of the instruction (in NASM)
    """
    name: String!
    """
    Operands of the instruction
    """
    operands: [String]
    """
    Length of the instruction in bytes
    """
    length: Int!
}
```

1. In `src/adapter/entrypoints.rs` I would expect the type the query is
implemented on to be present in the function args to run the query on the type.
But all I have is the query args and a `ResolveInfo` type. Maybe this is where I
get the data from? Upon looking further I see this is just called from the
`Adapter` impl it generates - so not sure why not just the todo's in the
`Adapter` and let me split out to my own free functions if I want.

There is also `src/adapter/vertex.rs`. This contains an enum for the vertex
types that queries can contain. The enum is empty of data so I need to
actually add the data types in. 

1. It would be good for the generated code to feature some TODO comments on
where you should change things
2. The readme could also link to a trustfall adapter implemented where stubgen
was used for the initial code. Just so people can compare the generated code
with the finished code.

Initially I thought I'd want to implement `Adapter` on my type where I put
the deserialized object files into, and then I can use the `&self` methods to
get the instructions and elf sections and make the query magic happen. I think
this approach is still right but having some comments in the generated code
would help. I guess the default approach is adapters on APIs to glue all those
data sources together rather than static assets already on the filesystem that
we may load.

Calling the adapter type adapter all the time seems like a name collision faff
if I was to bring in other adapter crates for different data souces :thinking:

## Implementing

### Vertex

Doing the vertex implementation is fairly straightforward, I put my crate
internal types into the enum as fields. One thing not mentioned is that
making the types cheap to clone `Rc` is recognised - although this is
strongly signalled by the enum deriving clone and not copy.

### Properties

The properties seems the easiest to implement, though for each field
I ended up crafting a lambda and mapping it in the iterator. So I make a lambda
like so:

```rust
|v: DataContext<V>| match v.active_vertex() {
    Some(Vertex::Expected(e) => (v.clone(), FieldValue::Ty(e.val)),
    None => (v, FieldValue::Null),
    Some(v) => unreachable!("Incorrect vertex: {:?}", v),
}
```

And then taking the `ContextIterator<'a, V>` running:

```rust
Box::new(contexts.map(func))
```

We can see my initial implementation of the `SourceLocation` property below:

```rust
pub(super) fn resolve_source_location_property<'a, V: AsVertex<Vertex> + 'a>(
    contexts: ContextIterator<'a, V>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, V, FieldValue> {
    let func = match property_name {
        "column" => |v: DataContext<V>| match v.active_vertex() {
            Some(Vertex::SourceLocation(loc)) => (v.clone(), FieldValue::Uint64(loc.column as u64)),
            None => (v, FieldValue::Null),
            Some(vertex) => unreachable!("Invalid vertex: {:?}", vertex),
        },
        "file" => |v: DataContext<V>| match v.active_vertex() {
            Some(Vertex::SourceLocation(loc)) => (
                v.clone(),
                FieldValue::String(Arc::from(loc.file.display().to_string().as_str())),
            ),
            None => (v, FieldValue::Null),
            Some(vertex) => unreachable!("Invalid vertex: {:?}", vertex),
        },
        "line" => |v: DataContext<V>| match v.active_vertex() {
            Some(Vertex::SourceLocation(loc)) => (v.clone(), FieldValue::Uint64(loc.line as u64)),
            None => (v, FieldValue::Null),
            Some(vertex) => unreachable!("Invalid vertex: {:?}", vertex),
        },
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'SourceLocation'"
            )
        }
    };
    Box::new(contexts.map(func))
}
```

I'm not yet sure if this is the correct approach, but that's more noobiness with
trustfall. It seems reasonable, though I'm sure I could simplify the code a bit
further.

Oh implementing all of these causes the tests to pass despite me not
implementing the queries. This makes sense as I was wondering just how deep the
invariant test can go as it can't feasibly test query behaviour. Maybe smart to
document this very explicitly. That way anyone implementing their queries first
then properties doesn't think they've got more guarantees from the tests than
they really do. Oh I can put a TODO in one of my match arms and it passes the
test so the properties don't even have to be fully implemented! Yeah good to
note this more in depth.

Right all properties implemented. It would be nice to have a view on how I can
test these more thoroughly but my feeling right now is that it's better to
just do some integration tests in the form of actual queries.

### Adapter

The lifetime and returning an iterator is a bit of a pain when implementing
adapter but collectubg a vec of results and then returning 
`Box::new(res.into_iter())` seems to work well enough. If I really fiddle
with it I could possibly make it lazier, but it might be hard as the result
lifetime is different to the self lifetime.

## Using it 

### Simple Test Program

So it would be nice if `trustfall::execute_query` had some example code in the
docs. Not sure how many people are using trustfall from Rust code but it seems
there's some gaps there (unless I'm just missing something in the docs).

Okay so here's my first attempt (I do not know GraphQL so this is very much me
feeling my way around it):

```rust
fn main() -> anyhow::Result<()> {
    let object = Arc::new(Adapter::load("target/debug/examples/basic")?);

    let query = "
        query Query($file: String) {
            getFileLocations(file: $file) {
                line,
            }
        }
        ";

    let variables = [("file", FieldValue::String("basic.rs".into()))].into_iter().collect();

    let result = execute_query(Adapter::schema(), object, query, variables).unwrap();
   
    let lines = result.collect::<Vec<_>>();
    println!("Basic.rs lines: {:?}", lines);
    Ok(())
}
```

And I got:

```
thread 'main' panicked at examples/basic.rs:20:77:
called `Result::unwrap()` on an `Err` value: Input contains multiple operation blocks, this is not supported

Caused by:
    Input contains multiple operation blocks, this is not supported
```

Remove the outer block and do:

```rust
let query = "
    {
        getFileLocations(file: String) {
            line,
        }
    }
    ";
```

And our error changes to:

```
thread 'main' panicked at /home/xd009642/.cargo/registry/src/index.crates.io-6f17d22bba15001f/trustfall_core-0.7.1/src/ir/types/base.rs:380:17:
not implemented: enum values are not currently supported: String! Enum("String")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Okay my syntax is clearly a bit wonky. Lets backtrack and start from a
hardcoded value:

```
    let query = "
        {
            getFileLocations(file: \"examples/basic.rs\") {
                line,
            }
        }
        ";
```

This looks a bit better but we get:

```
Basic.rs lines: [{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}]
```

Which is a bit odd...

I later found I was missing an `@output` tag so the query should have been `line @output` instead of just `line`.

With that solved I went onto my next challenge.

### Implementing addr2line

Here I was determined to get input arguments to a query working so I got to work with an initial pass of:

```rust
fn main() {
    let file = std::env::args().nth(1).expect("no object file given");
    let addr = std::env::args().nth(2).expect("no address given").parse::<u64>().expect("invalid address given");

    let object = Arc::new(Adapter::load(file).expect("Couldn't load file"));

    let query = "
        {
            getLocation(address: \"$addr\") {
                file @output,
                line @output,
                column @output,
            }
        }
        ";


    let variables = [("addr", FieldValue::Int64(addr as i64))].into_iter().collect();

    let result = execute_query(Adapter::schema(), object.clone(), query, variables).unwrap();
   
    let lines = result.collect::<Vec<_>>();
    if lines.is_empty() {
        panic!("No line for given address");
    } else {
        println!("{:?}", lines[0]);
    }
}
```

I did arrive on the quotes around addr after some experimentation but it's wrong. 

```
called `Result::unwrap()` on an `Err` value: Invalid value for edge parameter address on edge getLocation. Expected a value of type Int!, but got: String("$addr")
```

It complains the type is a string. Removing the quotes however gives me an
invalid value:

```
thread 'main' panicked at examples/addr2line.rs:29:85:
called `Result::unwrap()` on an `Err` value: Field getLocation received an invalid value for argument address: $addr

Caused by:
    Field getLocation received an invalid value for argument address: $addr
```

In talking with Predrag

> Ah, GraphQL variables don't work
> You have to use literals for edge parameters
> That's an open to-do item, and a surprising amount of work to get right

Okay so with a simple change to use `format!` to generate a query string with
the address in we now have a working query!

## Future Challenges

I'd like to be able to query a function in an executable and get a control flow
graph. This would involve querying a sequence of instructions and generating a
graph from it. 

> How does `trustfall_stubgen` handle me updating the schema and queries. Will
> it just wipe out my code?

## Thoughts on Iterative Updates

Now I'm updating the schema and looking to update my code there's a thought I
have. It would be nice if I could specify my existing generated adapter with the
impl filled in and have it genertae some stuff in place.

If I just add a new query and don't change anything else then that's just an
addition to the match arm in the adapter impl and maybe some extra things in
some of the other files. 

For changing existing queries (adding new args etc), I'm wondering on what a
simple approach could be. Generating the source files and generating a
diff for merging them into the current implementation might be nice. It could
fit into peoples existing editor flows for handling merge conflicts. Git not
knowing it's in a merge-conflict-esque state might just lead to people missing
some of the diffs and doing dumb auto commits of inline diffs maybe? 

Of course using syn we can parse the existing files and take the generated code
(assuming it's generated via something in the syn/quote ecosystem already) and
implement some sort of syn visitor to go over and try to smart merge things in
a structurally aware manner. This would take substantial implementation effort
in a library API that can be described as painful at best.
