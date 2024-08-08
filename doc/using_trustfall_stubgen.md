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

## Adapter

The lifetime and returning an iterator is a bit of a pain when implementing
adapter but collectubg a vec of results and then returning 
`Box::new(res.into_iter())` seems to work well enough. If I really fiddle
with it I could possibly make it lazier, but it might be hard as the result
lifetime is different to the self lifetime.
