use redis::*;

const GLOBAL_DBINDEX_NAME: &str = "GVM.__GlobalDBIndex";
const REDIS_DEFAULT_PATH: &str = "unix:///run/redis/redis-server.sock";
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

struct NvtCache {
    cache: RedisCtx,
    init: bool,
}


struct RedisCtx {
    kb: Connection, //a redis connection
    db: u32,    // the name space
    maxdb: u32 // max db index
}

impl RedisCtx {
    fn new() -> Result<RedisCtx, RedisError> {
        let client = redis::Client::open(REDIS_DEFAULT_PATH)?;
        let kb = client.get_connection()?;
        Ok(RedisCtx {kb: kb, db: 0, maxdb: 0})
    }

    fn max_db_index(&mut self) -> Result<u32, RedisError> {
        if self.maxdb > 0 {
            return Ok(self.maxdb)
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
                return Ok(self.maxdb)
            }
            Err(e) => {return Err(e)}
        }
        fn max_db_index_to_uint (res: Vec<String>) -> u32 {
            if res.len() == 2 {
                match res[1].to_string().parse::<u32>() {
                    Ok(m) => {return m}
                    Err(e) => {
                        println! ("{}",e);
                        return 0 as u32
                    }
                }
            }
            return 0 as u32
        }
    }
    
    fn redis_set_key_int(&mut self, key: &str, val: i32 ) -> Result<(), RedisError>{
        let _ : () = self.kb.set (key, val)?;
        Ok(())
    }       
 
    fn redis_get_int(&mut self, key: &str) -> String {
        match self.kb.get(key) {
            Ok(x) => x,
            Err(x) => { panic! ("{}", x) }
        }
        
    }       
}

/// NvtCache implementation.
impl NvtCache {
    /// initialize the NVT Cache. 
    fn init() -> Result<NvtCache, RedisError> {
        let rctx = RedisCtx::new()?;
        Ok(NvtCache {cache: rctx, init: true})

    }

    fn is_init (&mut self) -> bool {
        self.init == true
    }
}

//test
fn main() {
    let mut nvtcache: NvtCache;
    let n = NvtCache::init();
    match n {
        Ok(nc) => {nvtcache = nc}
        Err(e) => {panic!("Error: {}",e)}
    }
    match nvtcache.cache.max_db_index() {
        Ok(n) => println!("MAX: {}",n),
        Err(e) => println!("Error:{}",e)
    }
    println!("Accessing the struct: {}", nvtcache.cache.maxdb);

    match nvtcache.cache.max_db_index() {
        Ok(n) => println!("MAX: {}",n),
        Err(e) => println!("Error:{}",e)
    }
    if nvtcache.is_init() {
        println! ("Is initialized");
    }

}
