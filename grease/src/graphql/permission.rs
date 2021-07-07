use async_graphql::Guard;

pub struct Permission {
    name: &'static str,
    event_type: Option<&'static str>,
}

impl Permission {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            event_type: None,
        }
    }

    pub fn for_type(self, event_type: &'static str) -> Self {
        Self {
            name,
            event_type: Some(event_type),
        }
    }

    pub fn able_to(&self, member: &Member) -> bool {
        member.permissions().iter().any(|permission| {
            permission.name == self.name
                && (permission.event_type.is_none()
                    || permission.event_type.as_ref().map(|type_| type_.as_str()) == event_type)
        })
    }

    pub const PROCESS_GIG_REQUESTS: Self = Self::new("process-gig-requests");
    pub const PROCESS_ABSENCE_REQUESTS: Self = Self::new("process-absence-requests");

    pub const EDIT_REPERTOIRE: Self = Self::new("edit-repertoire");

    pub const EDIT_LINKS: Self = Self::new("edit-links");
    pub const EDIT_UNIFORMS: Self = Self::new("edit-uniforms");

    pub const EDIT_SEMESTER: Self = Self::new("edit-semester");
    pub const EDIT_PERMISSIONS: Self = Self::new("edit-permissions");
    pub const EDIT_OFFICERS: Self = Self::new("edit-officers");

    pub const VIEW_TRANSACTIONS: Self = Self::new("view-transactions");
    pub const EDIT_TRANSACTION: Self = Self::new("edit-transaction");

    pub const ADD_MULTI_TODO: Self = Self::new("add-multi-todo");

    pub const EDIT_MINUTES: Self = Self::new("edit-minutes");
    pub const VIEW_COMPLETE_MINUTES: Self = Self::new("view-complete-minutes");

    pub const EDIT_USER: Self = Self::new("edit-user");
    pub const SWITCH_USER: Self = Self::new("switch-user");
    pub const DELETE_USER: Self = Self::new("delete-user");
    pub const VIEW_USERS: Self = Self::new("view-users");
    pub const VIEW_USER_PRIVATE_DETAILS: Self = Self::new("view-user-private-details");

    pub const CREATE_EVENT: Self = Self::new("create-event");
    pub const MODIFY_EVENT: Self = Self::new("modify-event");
    pub const EDIT_ALL_EVENTS: Self = Self::new("edit-all-events");
    pub const DELETE_EVENT: Self = Self::new("delete-event");

    pub const EDIT_ATTENDANCE: Self = Self::new("edit-attendance");
    pub const EDIT_ATTENDANCE_OWN_SECTION: Self = Self::new("edit-attendance-own-section");

    pub const EDIT_CARPOOLS: Self = Self::new("edit-carpool");
}

impl Guard for Permission {
    fn guard(&self, ctx: Context<'_>) -> Result<()> {
        if let Some(user) = ctx.data_opt::<Member>() {
            if self.able_to(user) {
                return Ok(());
            }
        }

        Err(format!("Permission {} required", self.name))
    }
}
