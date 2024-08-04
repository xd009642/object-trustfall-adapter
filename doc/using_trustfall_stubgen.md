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
