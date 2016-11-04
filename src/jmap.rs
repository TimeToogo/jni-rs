use errors::*;

use jnienv::JNIEnv;
use jobject::JObject;
use jclass::JClass;
use jmethodid::JMethodID;

use signature::JavaType;
use signature::Primitive;

pub struct JMap<'a> {
    internal: JObject<'a>,
    class: JClass<'a>,
    get: JMethodID<'a>,
    put: JMethodID<'a>,
    remove: JMethodID<'a>,
    env: &'a JNIEnv<'a>,
}

impl<'a> ::std::ops::Deref for JMap<'a> {
    type Target = JObject<'a>;

    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}

impl<'a> From<JMap<'a>> for JObject<'a> {
    fn from(other: JMap) -> JObject {
        other.internal
    }
}

impl<'a> JMap<'a> {
    pub fn from_env(env: &'a JNIEnv<'a>, obj: JObject<'a>) -> Result<JMap<'a>> {
        let class = try!(env.find_class("java/util/Map"));

        let get = try!(env.get_method_id((class,
                                          "get",
                                          "(Ljava/lang/Object;\
                                           )Ljava/lang/Object;")));
        let put = try!(env.get_method_id((class,
                                          "put",
                                          "(Ljava/lang/Object;\
                                           Ljava/lang/Object;\
                                           )Ljava/lang/Object;")));

        let remove = try!(env.get_method_id((class,
                                             "remove",
                                             "(Ljava/lang/Object;\
                                              )Ljava/lang/Object;")));

        Ok(JMap {
            internal: obj,
            class: class,
            get: get,
            put: put,
            remove: remove,
            env: env,
        })
    }

    pub fn get(&self, key: JObject<'a>) -> Result<Option<JObject>> {
        let result = unsafe {
            self.env.call_method_unsafe(self.internal,
                                        self.get,
                                        JavaType::Object("java/lang/Object"
                                            .into()),
                                        &[key.into()])
        };

        match result {
            Ok(val) => Ok(Some(try!(val.l()))),
            Err(e) => {
                match e.kind() {
                    &ErrorKind::NullPtr(_) => Ok(None),
                    _ => Err(e),
                }
            }
        }
    }

    pub fn put(&self,
               key: JObject<'a>,
               value: JObject<'a>)
               -> Result<Option<JObject>> {
        let result = unsafe {
            self.env.call_method_unsafe(self.internal,
                                        self.put,
                                        JavaType::Object("java/lang/Object"
                                            .into()),
                                        &[key.into(), value.into()])
        };

        match result {
            Ok(val) => Ok(Some(try!(val.l()))),
            Err(e) => {
                match e.kind() {
                    &ErrorKind::NullPtr(_) => Ok(None),
                    _ => Err(e),
                }
            }
        }
    }

    pub fn remove(&self, key: JObject<'a>) -> Result<Option<JObject<'a>>> {
        let result = unsafe {
            self.env.call_method_unsafe(self.internal,
                                        self.remove,
                                        JavaType::Object("java/lang/Object"
                                            .into()),
                                        &[key.into()])
        };

        match result {
            Ok(val) => Ok(Some(try!(val.l()))),
            Err(e) => {
                match e.kind() {
                    &ErrorKind::NullPtr(_) => Ok(None),
                    _ => Err(e),
                }
            }
        }
    }

    pub fn iter(&'a self) -> Result<JMapIter<'a>> {
        let set = unsafe {
            let set = try!(self.env
                .call_method_unsafe(self.internal,
                                    (self.class,
                                     "entrySet",
                                     "()Ljava/util/Set;"),
                                    JavaType::Object("java/util/Set".into()),
                                    &[]));
            try!(set.l())
        };

        let iter = unsafe {
            let iter = try!(self.env
                .call_method_unsafe(set,
                                    ("java/util/Set",
                                     "iterator",
                                     "()Ljava/util/Iterator;"),
                                    JavaType::Object("java/util/Iterator"
                                        .into()),
                                    &[]));
            try!(iter.l())
        };

        let iter_class = try!(self.env
            .find_class("java/util/Iterator"));

        let has_next = try!(self.env
            .get_method_id((iter_class, "hasNext", "()Z")));

        let next = try!(self.env
            .get_method_id((iter_class, "next", "()Ljava/lang/Object;")));

        let entry_class = try!(self.env
            .find_class("java/util/Map$Entry"));

        let get_key = try!(self.env
            .get_method_id((entry_class, "getKey", "()Ljava/lang/Object;")));

        let get_value = try!(self.env
            .get_method_id((entry_class, "getValue", "()Ljava/lang/Object;")));

        Ok(JMapIter {
            map: &self,
            has_next: has_next,
            next: next,
            get_key: get_key,
            get_value: get_value,
            iter: iter,
        })
    }
}

pub struct JMapIter<'a> {
    map: &'a JMap<'a>,
    has_next: JMethodID<'a>,
    next: JMethodID<'a>,
    get_key: JMethodID<'a>,
    get_value: JMethodID<'a>,
    iter: JObject<'a>,
}

impl<'a> JMapIter<'a> {
    fn get_next(&self) -> Result<Option<(JObject<'a>, JObject<'a>)>> {
        let has_next = unsafe {
            let val = try!(self.map
                .env
                .call_method_unsafe(self.iter,
                                    self.has_next,
                                    JavaType::Primitive(Primitive::Boolean),
                                    &[]));
            try!(val.z())
        };

        if !has_next {
            return Ok(None);
        }
        let next = unsafe {
            let next = try!(self.map
                .env
                .call_method_unsafe(self.iter,
                                    self.next,
                                    JavaType::Object("java/util/Map$Entry"
                                        .into()),
                                    &[]));
            try!(next.l())
        };

        let key = unsafe {
            let key = try!(self.map
                .env
                .call_method_unsafe(next,
                                    self.get_key,
                                    JavaType::Object("java/lang/Object"
                                        .into()),
                                    &[]));
            try!(key.l())
        };

        let value = unsafe {
            let value = try!(self.map
                .env
                .call_method_unsafe(next,
                                    self.get_value,
                                    JavaType::Object("java/lang/Object"
                                        .into()),
                                    &[]));
            try!(value.l())
        };

        Ok(Some((key, value)))
    }
}

impl<'a> Iterator for JMapIter<'a> {
    type Item = (JObject<'a>, JObject<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.get_next() {
            Ok(Some(n)) => Some(n),
            _ => None,
        }
    }
}