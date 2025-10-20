use crate::error::{Result, SednaError};
use crate::ffi::*;
use std::ffi::{CStr, CString};

pub struct SednaClient {
    conn: SednaConnection,
}

impl SednaClient {
    pub fn connect(host: &str, port: u16, db_name: &str, login: &str, password: &str) -> Result<Self> {
        let mut conn = SednaConnection::default();

        let host_url = if port == 5050 {
            CString::new(host)?
        } else {
            CString::new(format!("{}:{}", host, port))?
        };
        let db_name_c = CString::new(db_name)?;
        let login_c = CString::new(login)?;
        let password_c = CString::new(password)?;

        let result = unsafe {
            SEconnect(
                &mut conn,
                host_url.as_ptr(),
                db_name_c.as_ptr(),
                login_c.as_ptr(),
                password_c.as_ptr(),
            )
        };

        if result != SEDNA_SESSION_OPEN {
            let error_msg = unsafe {
                let msg_ptr = SEgetLastErrorMsg(&mut conn);
                if msg_ptr.is_null() {
                    "Unknown error".to_string()
                } else {
                    CStr::from_ptr(msg_ptr).to_string_lossy().to_string()
                }
            };
            return Err(SednaError::ConnectionFailed(error_msg));
        }

        Ok(Self { conn })
    }

    pub fn execute(&mut self, query: &str) -> Result<QueryResult<'_>> {
        let query_c = CString::new(query)?;

        let result = unsafe { SEexecute(&mut self.conn, query_c.as_ptr()) };

        if result == SEDNA_QUERY_SUCCEEDED || result == SEDNA_UPDATE_SUCCEEDED {
            Ok(QueryResult { client: self })
        } else {
            let error_msg = unsafe {
                let msg_ptr = SEgetLastErrorMsg(&mut self.conn);
                if msg_ptr.is_null() {
                    "Unknown error".to_string()
                } else {
                    CStr::from_ptr(msg_ptr).to_string_lossy().to_string()
                }
            };
            Err(SednaError::QueryFailed(error_msg))
        }
    }

    pub fn begin_transaction(&mut self) -> Result<()> {
        let result = unsafe { SEbegin(&mut self.conn) };

        if result != SEDNA_BEGIN_TRANSACTION_SUCCEEDED {
            let error_msg = self.get_last_error();
            return Err(SednaError::TransactionFailed(format!(
                "Begin failed: {}",
                error_msg
            )));
        }

        Ok(())
    }

    pub fn commit_transaction(&mut self) -> Result<()> {
        let result = unsafe { SEcommit(&mut self.conn) };

        if result != SEDNA_COMMIT_TRANSACTION_SUCCEEDED {
            let error_msg = self.get_last_error();
            return Err(SednaError::TransactionFailed(format!(
                "Commit failed: {}",
                error_msg
            )));
        }

        Ok(())
    }

    pub fn rollback_transaction(&mut self) -> Result<()> {
        let result = unsafe { SErollback(&mut self.conn) };

        if result != SEDNA_ROLLBACK_TRANSACTION_SUCCEEDED {
            let error_msg = self.get_last_error();
            return Err(SednaError::TransactionFailed(format!(
                "Rollback failed: {}",
                error_msg
            )));
        }

        Ok(())
    }

    /// Load XML data from a string into a document.
    /// This is the programmatic equivalent of LOAD STDIN.
    /// If collection_name is None, creates a standalone document.
    pub fn load_xml_data(&mut self, xml_data: &str, doc_name: &str, collection_name: Option<&str>) -> Result<()> {
        let doc_name_c = CString::new(doc_name)?;
        let col_name_c = collection_name.map(|s| CString::new(s)).transpose()?;
        let col_ptr = col_name_c.as_ref().map_or(std::ptr::null(), |c| c.as_ptr());

        // Load the XML data
        let result = unsafe {
            SEloadData(
                &mut self.conn,
                xml_data.as_ptr() as *const i8,
                xml_data.len() as i32,
                doc_name_c.as_ptr(),
                col_ptr,
            )
        };

        if result != SEDNA_DATA_CHUNK_LOADED && result != SEDNA_BULK_LOAD_SUCCEEDED {
            let error_msg = self.get_last_error();
            let error_code = unsafe { SEgetLastErrorCode(&mut self.conn) };
            return Err(SednaError::QueryFailed(format!(
                "Load data failed (code {}): {}",
                error_code, error_msg
            )));
        }

        // End the bulk load
        let result = unsafe { SEendLoadData(&mut self.conn) };

        if result != SEDNA_BULK_LOAD_SUCCEEDED {
            let error_msg = self.get_last_error();
            return Err(SednaError::QueryFailed(format!(
                "End load data failed: {}",
                error_msg
            )));
        }

        Ok(())
    }

    fn get_last_error(&mut self) -> String {
        unsafe {
            let msg_ptr = SEgetLastErrorMsg(&mut self.conn);
            if msg_ptr.is_null() {
                "Unknown error".to_string()
            } else {
                CStr::from_ptr(msg_ptr).to_string_lossy().to_string()
            }
        }
    }
}

impl Drop for SednaClient {
    fn drop(&mut self) {
        unsafe {
            SEclose(&mut self.conn);
        }
    }
}

pub struct QueryResult<'a> {
    client: &'a mut SednaClient,
}

impl<'a> QueryResult<'a> {
    pub fn next(&mut self) -> Result<Option<String>> {
        let result = unsafe { SEnext(&mut self.client.conn) };

        match result {
            SEDNA_RESULT_END => Ok(None),
            SEDNA_NEXT_ITEM_SUCCEEDED => {
                // Read the data
                let mut buffer = vec![0u8; 1024 * 1024]; // 1MB buffer
                let mut full_data = Vec::new();

                loop {
                    let bytes_read = unsafe {
                        SEgetData(
                            &mut self.client.conn,
                            buffer.as_mut_ptr() as *mut i8,
                            buffer.len() as i32,
                        )
                    };

                    if bytes_read < 0 {
                        let error_msg = self.client.get_last_error();
                        return Err(SednaError::QueryFailed(format!(
                            "Failed to get data: {}",
                            error_msg
                        )));
                    }

                    if bytes_read == 0 {
                        break;
                    }

                    full_data.extend_from_slice(&buffer[..bytes_read as usize]);
                }

                let result_str = String::from_utf8_lossy(&full_data).to_string();
                Ok(Some(result_str))
            }
            _ => {
                let error_msg = self.client.get_last_error();
                Err(SednaError::QueryFailed(error_msg))
            }
        }
    }

    pub fn collect_all(&mut self) -> Result<Vec<String>> {
        let mut results = Vec::new();
        while let Some(item) = self.next()? {
            results.push(item);
        }
        Ok(results)
    }
}
