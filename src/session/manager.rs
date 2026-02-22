use dashmap::DashMap;
use tokio::sync::mpsc;
use crate::session::state::{SessionState, SessionHandle};
use std::sync::Arc;

pub struct SessionManager {
    sessions: Arc<DashMap<String, SessionState>>,
    handles: Arc<DashMap<String, SessionHandle>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            handles: Arc::new(DashMap::new()),
        }
    }

    pub async fn create_session(&self, client_id: String, clean_session: bool, keep_alive: u16) -> SessionState {
        let session = SessionState::new(client_id.clone(), clean_session, keep_alive);
        
        let (sender, _receiver) = mpsc::channel::<Vec<u8>>(100);
        let handle = SessionHandle::new(client_id.clone(), sender);
        
        self.sessions.insert(client_id.clone(), session.clone());
        self.handles.insert(client_id.clone(), handle);
        
        session
    }

    pub async fn get_session(&self, client_id: &str) -> Option<SessionState> {
        self.sessions.get(client_id).map(|s| s.clone())
    }

    pub async fn get_handle(&self, client_id: &str) -> Option<SessionHandle> {
        self.handles.get(client_id).map(|h| SessionHandle {
            client_id: h.client_id.clone(),
            sender: h.sender.clone(),
        })
    }

    pub async fn subscribe(&self, client_id: &str, topic: String, qos: u8) -> Option<()> {
        if let Some(mut session) = self.sessions.get_mut(client_id) {
            session.subscribe(topic, qos);
            session.update_activity();
            Some(())
        } else {
            None
        }
    }

    pub async fn unsubscribe(&self, client_id: &str, topic: &str) -> Option<()> {
        if let Some(mut session) = self.sessions.get_mut(client_id) {
            session.unsubscribe(topic);
            session.update_activity();
            Some(())
        } else {
            None
        }
    }

    pub async fn update_activity(&self, client_id: &str) -> Option<()> {
        if let Some(mut session) = self.sessions.get_mut(client_id) {
            session.update_activity();
            Some(())
        } else {
            None
        }
    }

    pub async fn disconnect(&self, client_id: &str) -> Option<()> {
        if let Some(session) = self.sessions.get(client_id) {
            if session.clean_session {
                self.sessions.remove(client_id);
                self.handles.remove(client_id);
            } else {
                if let Some(mut s) = self.sessions.get_mut(client_id) {
                    s.connected = false;
                }
            }
            Some(())
        } else {
            None
        }
    }

    pub async fn remove_session(&self, client_id: &str) {
        self.sessions.remove(client_id);
        self.handles.remove(client_id);
    }

    pub fn get_all_sessions(&self) -> Vec<SessionState> {
        self.sessions.iter().map(|s| s.clone()).collect()
    }

    pub fn get_session_count(&self) -> usize {
        self.sessions.len()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
