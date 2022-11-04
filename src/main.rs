use redis::*;
use std::error;
use std::fmt;


const GLOBAL_DBINDEX_NAME: &str = "GVM.__GlobalDBIndex";
const REDIS_DEFAULT_PATH: &str = "unix:///run/redis-openvas/redis.sock";
const NVTCACHE: &str = "nvticache";

enum KbNvtPos {
    NvtFilenamePos,
    NvtRequiredKeysPos,
    NvtMandatoryKeysPos,
    NvtExcludedKeysPos,
    NvtRequiredUDPPortsPos,
    NvtRequiredPortsPos,
    NvtDependenciesPos,
    NvtTagsPos,
    NvtCvesPos,
    NvtBidsPos,
    NvtXrefsPos,
    NvtCategoryPos,
    NvtFamilyPos,
    NvtNamePos,
    NvtTimestampPos,
    NvtOIDPos,
}

type Result<T> = std::result::Result<T, DbError>;
 
#[derive(Debug)]
enum DbError {
    RedisErr(RedisError),
    CustomErr(String),
}
 
impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            DbError::RedisErr(..) => write!(f, "Redis Error"),
            DbError::CustomErr(e) => write!(f, "Error: String Message {}", e),
        }
    }
}
 
impl error::Error for DbError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            DbError::RedisErr(ref e) => Some(e),
            DbError::CustomErr(_) => None
        }
    }
}

impl From<RedisError> for DbError {
    fn from(err: RedisError) -> DbError {
        DbError::RedisErr(err)
    }
}

struct NvtCache {
    cache: RedisCtx,
    init: bool,
}

struct RedisCtx {
    kb: Connection, //a redis connection
    db: u32,        // the name space
    maxdb: u32,     // max db index
}

impl RedisCtx {
    fn new() -> Result<RedisCtx> {
        let client = redis::Client::open(REDIS_DEFAULT_PATH)?;
        let kb = client.get_connection()?;
        Ok(RedisCtx {
            kb,
            db: 0,
            maxdb: 0,
        })
    }

    fn max_db_index(&mut self) -> Result<u32> {
        if self.maxdb > 0 {
            return Ok(self.maxdb);
        }

        let maxdb = Cmd::new()
            .arg("CONFIG")
            .arg("GET")
            .arg("databases")
            .query(&mut self.kb);

        match maxdb {
            Ok(mdb) => {
                let res: Vec<String> = mdb;
                self.maxdb = max_db_index_to_uint(res);
                return Ok(self.maxdb);
            }
            Err(_) => return Err(DbError::CustomErr(String::from("Not possible to select a free database."))),
        }

        fn max_db_index_to_uint(res: Vec<String>) -> u32 {
            if res.len() == 2 {
                match res[1].to_string().parse::<u32>() {
                    Ok(m) => return m,
                    Err(e) => {
                        println!("{}", e);
                        return 0 as u32;
                    }
                }
            }
            return 0 as u32;
        }
    }

    fn set_namespace (&mut self, db_index: u32) -> Result<String> {
        let s = Cmd::new()
            .arg("SELECT")
            .arg(db_index.to_string())
            .query(&mut self.kb);

        match s {
            Ok(ok) => {
                self.db = db_index;
                return Ok(ok)
            },
            Err(_) => return Err(DbError::CustomErr(String::from("Not possible to set a namespace.")))
        }
    }


    fn try_database(&mut self, dbi: u32) -> Result<u32> {
        return Ok(1);
    }

    fn select_database(&mut self) -> Result<u32> {
        let maxdb: u32 = self.max_db_index()?;

        if self.db == 0 {
            for i in 1..maxdb {
                match self.try_database(i) {
                    Ok(selected_db) => {
                        self.db = selected_db;
                        match self.set_namespace(i) {
                            Ok(_) => (),
                            Err(e) => {
                                println!("Error: {}",e);
                                break;
                            }
                        }
                        return Ok(self.db)
                    },
                    Err(_) => continue,
                }
            }
        }
        return Err(DbError::CustomErr(String::from("Not possible to select a free database.")));
    }


    fn redis_set_key_int(&mut self, key: &str, val: i32) -> Result<()> {
        let _: () = self.kb.set(key, val)?;
        Ok(())
    }

    fn redis_get_int(&mut self, key: &str) -> String {
        match self.kb.get(key) {
            Ok(x) => {return x},
            Err(e) => e.to_string()
        }
    }
}

/// NvtCache implementation.
impl NvtCache {
    /// initialize the NVT Cache.
    fn init() -> Result<NvtCache> {
        let mut rctx = RedisCtx::new()?;
        let kbi = rctx.select_database()?;
        Ok(NvtCache {
            cache: rctx,
            init: true
        })
    }


    fn is_init(&mut self) -> bool {
        self.init == true
    }
}

//test
fn main() {
    let mut nvtcache: NvtCache;
    let n = NvtCache::init();
    match n {
        Ok(nc) => nvtcache = nc,
        Err(e) => {
            println!("{}", e);
            panic!("Error")
        }
    }
    match nvtcache.cache.max_db_index() {
        Ok(n) => println!("MAX: {}", n),
        Err(e) => println!("Error:{}", e),
    }
    println!("Accessing the struct: {}", nvtcache.cache.maxdb);

    if nvtcache.is_init() {
        println!("Is initialized");
    }

    match nvtcache.cache.set_namespace(1) {
        Ok(ok) => println!("Select 1 {}", ok),
        Err(e) => println!("Error:{}", e),
    }

    println!("The namespace: {}", nvtcache.cache.kb.get_db());
    println!("The namespace: {}", nvtcache.cache.db);

    let key = "key";
    let val = 42;
    match nvtcache.cache.redis_set_key_int(key, 42) {
        Ok(_) => println!("Key {} set with {}", key, val),
        Err(e) => println!("Error:{}", e),
    }
    println!("{}", nvtcache.cache.redis_get_int(key))
}
