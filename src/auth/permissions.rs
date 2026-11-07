use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Perm {
    // Phiếu khám
    RecordView,
    RecordCreate,
    RecordDelete,
    RecordExport,
    RecordConfirm,
    RecordUnlock,

    // Nhập liệu chuyên khoa
    FormHanhChinh,
    FormTienSu,
    ExamTheLuc,
    ExamNoi,
    ExamNgoai,
    ExamDaLieu,
    ExamSan,
    ExamMat,
    ExamTmh,
    ExamRhm,
    ExamCls,
    FormKetLuan,

    // Quản trị
    CatalogManage,
    Qd1613Manage,
    UserManage,
    RoleManage,
    SettingsManage,
}

impl Perm {
    pub fn as_str(&self) -> &'static str {
        match self {
            Perm::RecordView => "record.view",
            Perm::RecordCreate => "record.create",
            Perm::RecordDelete => "record.delete",
            Perm::RecordExport => "record.export",
            Perm::RecordConfirm => "record.confirm",
            Perm::RecordUnlock => "record.unlock",
            Perm::FormHanhChinh => "form.hanh_chinh",
            Perm::FormTienSu => "form.tien_su",
            Perm::ExamTheLuc => "exam.the_luc",
            Perm::ExamNoi => "exam.noi",
            Perm::ExamNgoai => "exam.ngoai",
            Perm::ExamDaLieu => "exam.da_lieu",
            Perm::ExamSan => "exam.san",
            Perm::ExamMat => "exam.mat",
            Perm::ExamTmh => "exam.tmh",
            Perm::ExamRhm => "exam.rhm",
            Perm::ExamCls => "exam.cls",
            Perm::FormKetLuan => "form.ket_luan",
            Perm::CatalogManage => "catalog.manage",
            Perm::Qd1613Manage => "qd1613.manage",
            Perm::UserManage => "user.manage",
            Perm::RoleManage => "role.manage",
            Perm::SettingsManage => "settings.manage",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "record.view" => Some(Perm::RecordView),
            "record.create" => Some(Perm::RecordCreate),
            "record.delete" => Some(Perm::RecordDelete),
            "record.export" => Some(Perm::RecordExport),
            "record.confirm" => Some(Perm::RecordConfirm),
            "record.unlock" => Some(Perm::RecordUnlock),
            "form.hanh_chinh" => Some(Perm::FormHanhChinh),
            "form.tien_su" => Some(Perm::FormTienSu),
            "exam.the_luc" => Some(Perm::ExamTheLuc),
            "exam.noi" => Some(Perm::ExamNoi),
            "exam.ngoai" => Some(Perm::ExamNgoai),
            "exam.da_lieu" => Some(Perm::ExamDaLieu),
            "exam.san" => Some(Perm::ExamSan),
            "exam.mat" => Some(Perm::ExamMat),
            "exam.tmh" => Some(Perm::ExamTmh),
            "exam.rhm" => Some(Perm::ExamRhm),
            "exam.cls" => Some(Perm::ExamCls),
            "form.ket_luan" => Some(Perm::FormKetLuan),
            "catalog.manage" => Some(Perm::CatalogManage),
            "qd1613.manage" => Some(Perm::Qd1613Manage),
            "user.manage" => Some(Perm::UserManage),
            "role.manage" => Some(Perm::RoleManage),
            "settings.manage" => Some(Perm::SettingsManage),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Perm::RecordView => "Xem hồ sơ",
            Perm::RecordCreate => "Tạo mới hồ sơ",
            Perm::RecordDelete => "Xóa hồ sơ",
            Perm::RecordExport => "Xuất CSV",
            Perm::RecordConfirm => "Ký / Khóa hồ sơ",
            Perm::RecordUnlock => "Mở khóa hồ sơ",
            Perm::FormHanhChinh => "Khám Hành chính",
            Perm::FormTienSu => "Khám Tiền sử",
            Perm::ExamTheLuc => "Khám Thể lực",
            Perm::ExamNoi => "Khám Nội khoa",
            Perm::ExamNgoai => "Khám Ngoại khoa",
            Perm::ExamDaLieu => "Khám Da liễu",
            Perm::ExamSan => "Khám Sản phụ khoa",
            Perm::ExamMat => "Khám Mắt",
            Perm::ExamTmh => "Khám Tai Mũi Họng",
            Perm::ExamRhm => "Khám Răng Hàm Mặt",
            Perm::ExamCls => "Khám Cận Lâm Sàng",
            Perm::FormKetLuan => "Kết luận sức khỏe",
            Perm::CatalogManage => "Quản lý danh mục",
            Perm::Qd1613Manage => "Quản lý QĐ 1613",
            Perm::UserManage => "Quản lý người dùng",
            Perm::RoleManage => "Phân quyền vai trò",
            Perm::SettingsManage => "Cài đặt hệ thống",
        }
    }

    pub fn all() -> &'static [Perm] {
        &[
            Perm::RecordView,
            Perm::RecordCreate,
            Perm::RecordDelete,
            Perm::RecordExport,
            Perm::RecordConfirm,
            Perm::RecordUnlock,
            Perm::FormHanhChinh,
            Perm::FormTienSu,
            Perm::ExamTheLuc,
            Perm::ExamNoi,
            Perm::ExamNgoai,
            Perm::ExamDaLieu,
            Perm::ExamSan,
            Perm::ExamMat,
            Perm::ExamTmh,
            Perm::ExamRhm,
            Perm::ExamCls,
            Perm::FormKetLuan,
            Perm::CatalogManage,
            Perm::Qd1613Manage,
            Perm::UserManage,
            Perm::RoleManage,
            Perm::SettingsManage,
        ]
    }
}

impl std::str::FromStr for Perm {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "record.view" => Ok(Perm::RecordView),
            "record.create" => Ok(Perm::RecordCreate),
            "record.delete" => Ok(Perm::RecordDelete),
            "record.export" => Ok(Perm::RecordExport),
            "record.confirm" => Ok(Perm::RecordConfirm),
            "record.unlock" => Ok(Perm::RecordUnlock),
            "form.hanh_chinh" => Ok(Perm::FormHanhChinh),
            "form.tien_su" => Ok(Perm::FormTienSu),
            "exam.the_luc" => Ok(Perm::ExamTheLuc),
            "exam.noi" => Ok(Perm::ExamNoi),
            "exam.ngoai" => Ok(Perm::ExamNgoai),
            "exam.da_lieu" => Ok(Perm::ExamDaLieu),
            "exam.san" => Ok(Perm::ExamSan),
            "exam.mat" => Ok(Perm::ExamMat),
            "exam.tmh" => Ok(Perm::ExamTmh),
            "exam.rhm" => Ok(Perm::ExamRhm),
            "exam.cls" => Ok(Perm::ExamCls),
            "form.ket_luan" => Ok(Perm::FormKetLuan),
            "catalog.manage" => Ok(Perm::CatalogManage),
            "qd1613.manage" => Ok(Perm::Qd1613Manage),
            "user.manage" => Ok(Perm::UserManage),
            "role.manage" => Ok(Perm::RoleManage),
            "settings.manage" => Ok(Perm::SettingsManage),
            _ => Err(()),
        }
    }
}
