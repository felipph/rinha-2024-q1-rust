use actix_postgres::bb8_postgres::tokio_postgres::Client;
use serde::Deserialize;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use bb8_postgres::bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use lazy_static::lazy_static;
use postgres::types::ToSql;
use postgres::NoTls;

lazy_static! {
    static ref CONNECTION_POOL: Pool<PostgresConnectionManager<NoTls>> = {
        let manager = PostgresConnectionManager::new_from_stringlike(
            "dbname=rinha host=database user=postgres",
            NoTls,
        )
        .unwrap();
        Pool::builder().max_size(10).build_unchecked(manager)
    };
}


#[get("/")]
async fn hello() -> impl Responder {
    // HttpResponse::Ok().body(format!("Number: {}", query("SELECT 1").await))
    HttpResponse::Ok().body("OK")
}

#[post("/clientes/{id}/transacoes")]
async fn transacoes(path: web::Path<i32>, transacao: web::Json<Transacao>) -> impl Responder {
    let cliente_id = path.into_inner();
    // println!("{cliente_id}");
    // println!("[{:?}]", transacao.valor);

    let c = CONNECTION_POOL.get().await.expect("Get a connection");
    let client: &Client = &*c;
    let temp = String::from(transacao.tipo);
    let tipo2 = temp.as_str();
    let valor = transacao.valor;
    let descricao = transacao.descricao.as_str();

    let row = client
        .query_one(
            "call do_trans($1, $2, $3, $4, '', 1, 1);",
            &[
                &cliente_id as &(dyn ToSql + Sync), 
                &tipo2 as &(dyn ToSql + Sync),
                &valor as &(dyn ToSql + Sync),
                &descricao as &(dyn ToSql + Sync)
                ],
                
        )
        .await
        .unwrap();
    // println!("{:?}", row.columns());
    let status = String::from(row.get::<_, &str>(0));
    if status == "200" {
        let saldo = row.get::<_, i32>(1);
        let limite = row.get::<_, i32>(2);    
        return HttpResponse::Ok().body(format!("{{ \"saldo\": {}, \"limite\": {} }}", saldo, limite))
    }
    if status == "404" {
        return HttpResponse::NotFound().body("");
    }
    return HttpResponse::UnprocessableEntity().body("");
}

#[get("/clientes/{id}/extrato")]
async fn extrato(path: web::Path<i32>) -> impl Responder {
    let cliente_id = path.into_inner();

    let c = CONNECTION_POOL.get().await.expect("Get a connection");
    let client: &Client = &*c;

    let row = client
        .query_one(
            "CALL DO_EXTRATO($1,'','');",
            &[&cliente_id as &(dyn ToSql + Sync)],
        )
        .await
        .unwrap();
    
    let status = String::from(row.get::<_, &str>(0));
    // println!("{:?}", row.columns());
    // println!("[{:?}]", status);

    if status == "200" {
        let response = String::from(row.get::<usize, &str>(1));
        return HttpResponse::Ok().body(response);
    } 
    if status == "404" {
        return HttpResponse::NotFound().body("Nao encontrado");
    }
    return HttpResponse::UnprocessableEntity().body("{}");

}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let c = envy::from_env::<Configuration>().expect("Please provide PORT env var");
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(extrato)
            .service(transacoes)
    })
    .bind(("127.0.0.1", c.port))?
    .run()
    .await
}

#[derive(Deserialize)]
struct Transacao {
    valor: i32,
    tipo: char,
    descricao: String
}


#[derive(Deserialize, Debug)]
struct Configuration {
    port: u16   
}