use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CatalogRecord {
    pub id: i64,
    pub cat_type: String,
    pub code: String,
    pub name: String,
    pub parent_code: String,
    pub sort_order: i32,
    pub is_active: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CatalogTypeInfo {
    pub cat_type: &'static str,
    pub title: &'static str,
    pub has_parent: bool,
    pub parent_type: Option<&'static str>,
}

impl CatalogTypeInfo {
    pub fn all() -> &'static [CatalogTypeInfo] {
        &[
            CatalogTypeInfo { cat_type: "priority", title: "1. Đối tượng (16 nhóm)", has_parent: false, parent_type: None },
            CatalogTypeInfo { cat_type: "payment", title: "2. Nguồn chi trả", has_parent: false, parent_type: None },
            CatalogTypeInfo { cat_type: "province", title: "3. Tỉnh / TP", has_parent: false, parent_type: None },
            CatalogTypeInfo { cat_type: "ward", title: "4. Xã / Phường", has_parent: true, parent_type: Some("province") },
            CatalogTypeInfo { cat_type: "ethnicity", title: "5. Dân tộc", has_parent: false, parent_type: None },
            CatalogTypeInfo { cat_type: "occupation", title: "6. Nghề nghiệp", has_parent: false, parent_type: None },
            CatalogTypeInfo { cat_type: "icd10", title: "7. Mã ICD-10", has_parent: false, parent_type: None },
            CatalogTypeInfo { cat_type: "blood_type", title: "8. Nhóm máu", has_parent: false, parent_type: None },
            CatalogTypeInfo { cat_type: "exam_reason", title: "9. Lý do khám", has_parent: false, parent_type: None },
            CatalogTypeInfo { cat_type: "kcb_type", title: "10. Loại hình KCB", has_parent: false, parent_type: None },
            CatalogTypeInfo { cat_type: "gender", title: "11. Giới tính", has_parent: false, parent_type: None },
            CatalogTypeInfo { cat_type: "id_issuer", title: "12. Nơi cấp CCCD", has_parent: false, parent_type: None },
            CatalogTypeInfo { cat_type: "conclusion_sample", title: "13. Kết luận mẫu", has_parent: false, parent_type: None },
        ]
    }
}

