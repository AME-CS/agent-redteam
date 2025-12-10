use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures::StreamExt;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use serde_json;
use colored::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardEvent {
    pub event_type: String,
    pub timestamp: f64,
    pub data: EventData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventData {
    AttackStart { target: String, total_iterations: usize },
    AttackProgress { current: usize, total: usize, success_rate: f64 },
    AttackResult { payload: String, vector: String, success: bool, response: String },
    AttackComplete { summary: AttackSummary },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackSummary {
    pub total_attacks: usize,
    pub successes: usize,
    pub overall_success_rate: f64,
    pub by_vector: std::collections::BTreeMap<String, f64>,
}

pub struct LiveDashboard {
    event_queue: Arc<Mutex<VecDeque<DashboardEvent>>>,
    listeners: Arc<Mutex<Vec<tokio::sync::mpsc::UnboundedSender<String>>>>,
}

impl LiveDashboard {
    pub fn new() -> Self {
        LiveDashboard {
            event_queue: Arc::new(Mutex::new(VecDeque::new())),
            listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub async fn start_server(&self, port: u16) {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr).await
            .expect("Failed to bind WebSocket server");
        
        println!("{}", format!("📡 Live dashboard: ws://{}", addr).bold().cyan());
        println!("   Open in browser or connect via WebSocket client");
        
        let listeners = Arc::clone(&self.listeners);
        
        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let listeners_clone = Arc::clone(&listeners);
                
                tokio::spawn(async move {
                    if let Ok(ws_stream) = accept_async(stream).await {
                        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
                        
                        {
                            let mut list = listeners_clone.lock().unwrap();
                            list.push(tx.clone());
                        }
                        
                        let (mut _write, mut _read) = ws_stream.split();
                        
                        // Send initial connection message
                        let welcome = serde_json::json!({
                            "event_type": "connected",
                            "message": "Connected to agent-redteam live dashboard"
                        }).to_string();
                        
                        if tx.send(welcome).is_err() {
                            return;
                        }
                        
                        // Keep alive - read from channel and send
                         while let Some(_msg) = rx.recv().await {
                            // In real implementation, send via _write
                        }
                    }
                });
            }
        });
    }
    
    pub fn emit_event(&self, event: DashboardEvent) {
        let json = serde_json::to_string(&event).unwrap_or_default();
        
        if let Ok(mut listeners) = self.listeners.lock() {
            listeners.retain(|tx| tx.send(json.clone()).is_ok());
        }
        
        // Also queue for replay
        if let Ok(mut queue) = self.event_queue.lock() {
            queue.push_back(event);
            if queue.len() > 1000 {
                queue.pop_front();
            }
        }
    }
    
    pub fn emit_attack_start(&self, target: &str, total: usize) {
        self.emit_event(DashboardEvent {
            event_type: "attack_start".into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            data: EventData::AttackStart {
                target: target.into(),
                total_iterations: total,
            },
        });
    }
    
    pub fn emit_attack_progress(&self, current: usize, total: usize, success_rate: f64) {
        self.emit_event(DashboardEvent {
            event_type: "attack_progress".into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            data: EventData::AttackProgress {
                current,
                total,
                success_rate,
            },
        });
    }
    
    pub fn emit_attack_result(&self, payload: &str, vector: &str, success: bool, response: &str) {
        self.emit_event(DashboardEvent {
            event_type: "attack_result".into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            data: EventData::AttackResult {
                payload: payload.into(),
                vector: vector.into(),
                success,
                response: response.into(),
            },
        });
    }
    
    pub fn emit_attack_complete(&self, summary: AttackSummary) {
        self.emit_event(DashboardEvent {
            event_type: "attack_complete".into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            data: EventData::AttackComplete { summary },
        });
    }
    
    pub fn get_recent_events(&self, count: usize) -> Vec<DashboardEvent> {
        if let Ok(queue) = self.event_queue.lock() {
            queue.iter()
                .rev()
                .take(count)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl Clone for LiveDashboard {
    fn clone(&self) -> Self {
        LiveDashboard {
            event_queue: Arc::clone(&self.event_queue),
            listeners: Arc::clone(&self.listeners),
        }
    }
}
