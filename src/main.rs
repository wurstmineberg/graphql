use {
    std::{
        io,
        path::Path,
    },
    async_graphql::{
        *,
        http::{
            GraphQLPlaygroundConfig,
            playground_source,
        },
    },
    async_graphql_rocket::{
        GraphQLQuery,
        GraphQLRequest,
        GraphQLResponse,
    },
    lazy_regex::regex_is_match,
    mcanvil::Region,
    rocket::{
        State,
        response::content::RawHtml,
    },
};

const WORLDS_DIR: &str = "/opt/wurstmineberg/world"; //TODO import from systemd-minecraft

type WmbSchema = Schema<Query, EmptyMutation, EmptySubscription>;

struct Query;

#[Object] impl Query {
    async fn world(&self, name: String) -> Option<World> {
        (regex_is_match!("^[0-9a-z]+$", &name) && Path::new(WORLDS_DIR).join(&name).exists()).then(|| World { name })
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
struct World {
    name: String,
}

#[ComplexObject] impl World {
    async fn dimension(&self, kind: DimensionKind) -> Dimension<'_> {
        Dimension {
            kind,
            world: self,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Enum)]
enum DimensionKind {
    Overworld,
    Nether,
    End,
}

#[derive(Clone, Copy, SimpleObject)]
#[graphql(complex)]
struct Dimension<'a> {
    world: &'a World,
    kind: DimensionKind,
}

#[derive(Debug, thiserror::Error)]
enum AnvilError {
    #[error(transparent)] ChunkColumn(#[from] mcanvil::ChunkColumnDecodeError),
    #[error(transparent)] Region(#[from] mcanvil::RegionDecodeError),
}

#[ComplexObject] impl<'a> Dimension<'a> {
    async fn chunk(
        &self,
        #[graphql(desc = "The chunk x coordinate, equivalent to the block x coordinates of the blocks in the chunk divided by 16")] cx: i32,
        #[graphql(desc = "The chunk y coordinate, equivalent to the block y coordinates of the blocks in the chunk divided by 16")] cy: i8,
        #[graphql(desc = "The chunk z coordinate, equivalent to the block z coordinates of the blocks in the chunk divided by 16")] cz: i32,
    ) -> Result<Option<Chunk<'_>>, AnvilError> {
        let world_data_path = Path::new(WORLDS_DIR).join(&self.world.name).join("world");
        match Region::open(match self.kind {
            DimensionKind::Overworld => world_data_path.join("region"),
            DimensionKind::Nether => world_data_path.join("DIM-1").join("region"),
            DimensionKind::End => world_data_path.join("DIM1").join("region"),
        }.join(format!("r.{}.{}.mca", cx.div_euclid(32), cz.div_euclid(32)))) { //TODO Region::open_async
            Ok(region) => {
                Ok(region.chunk_column([cx, cz])?.and_then(|col| col.into_section_at(cy)).map(|section| Chunk {
                    dimension: *self,
                    cx, cy, cz, section,
                }))
            }
            Err(mcanvil::RegionDecodeError::Io(e)) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
struct Chunk<'a> {
    dimension: Dimension<'a>,
    /// The chunk x coordinate, equivalent to the block x coordinates of the blocks in the chunk divided by 16
    cx: i32,
    /// The chunk y coordinate, equivalent to the block y coordinates of the blocks in the chunk divided by 16
    cy: i8,
    /// The chunk z coordinate, equivalent to the block z coordinates of the blocks in the chunk divided by 16
    cz: i32,
    #[graphql(skip)]
    section: mcanvil::ChunkSection,
}

#[ComplexObject] impl Chunk<'_> {
    async fn layers(&self) -> [ChunkLayer<'_>; 16] {
        self.section.blocks().map(|layer| ChunkLayer {
            rows: layer.map(|row| ChunkRow {
                blocks: row.map(|mcanvil::BlockState { name, .. }| Block { id: name }),
            }),
        })
    }
}

#[derive(SimpleObject)]
struct ChunkLayer<'a> {
    rows: [ChunkRow<'a>; 16],
}

#[derive(SimpleObject)]
struct ChunkRow<'a> {
    blocks: [Block<'a>; 16],
}

#[derive(SimpleObject)]
struct Block<'a> {
    id: &'a str,
}

#[rocket::get("/")] //TODO move to /graphql/playground or similar?
fn graphql_playground() -> RawHtml<String> {
    RawHtml(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

#[rocket::get("/graphql?<query..>")]
async fn graphql_query(schema: &State<WmbSchema>, query: GraphQLQuery) -> GraphQLResponse {
    query.execute(schema).await
}

#[rocket::post("/graphql", data = "<request>", format = "application/json")]
async fn graphql_request(schema: &State<WmbSchema>, request: GraphQLRequest) -> GraphQLResponse {
    request.execute(schema).await
}

#[rocket::launch]
fn rocket() -> _ {
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .finish();
    rocket::custom(rocket::Config {
        port: 24811,
        ..rocket::Config::default()
    })
    .manage(schema)
    .mount("/", rocket::routes![
        graphql_query,
        graphql_request,
        graphql_playground,
    ])
}
