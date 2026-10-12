//! Static Mock Data Service for Demo Showcase

use crate::pages::demo::models::MockUserItem;

pub struct DemoMockService;

impl DemoMockService {
    /// Provide static list of mock users for tables and avatar demonstrations.
    pub fn get_mock_users() -> Vec<MockUserItem> {
        vec![
            MockUserItem {
                id: "USR-001",
                name: "สมชาย สายลม",
                email: "somchai@arch.local",
                role: "Super Admin",
                department: "ฝ่ายเทคโนโลยีสารสนเทศ",
                status: "Active",
            },
            MockUserItem {
                id: "USR-002",
                name: "วิภาดา รักดี",
                email: "wiphada@arch.local",
                role: "HR Manager",
                department: "ฝ่ายทรัพยากรบุคคล",
                status: "Active",
            },
            MockUserItem {
                id: "USR-003",
                name: "กิตติพงษ์ มั่นคง",
                email: "kittipong@arch.local",
                role: "Finance Lead",
                department: "ฝ่ายบัญชีและการเงิน",
                status: "Pending",
            },
            MockUserItem {
                id: "USR-004",
                name: "นภาพร สดใส",
                email: "naphaporn@arch.local",
                role: "System Auditor",
                department: "ฝ่ายตรวจสอบภายใน",
                status: "Inactive",
            },
        ]
    }
}
