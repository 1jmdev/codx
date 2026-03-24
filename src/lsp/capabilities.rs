use lsp_types::{
    ClientCapabilities, CodeActionProviderCapability, DeclarationCapability,
    HoverProviderCapability, ImplementationProviderCapability, InitializeResult, OneOf,
    ServerCapabilities, TypeDefinitionProviderCapability,
};

#[derive(Debug, Clone)]
pub struct NegotiatedCapabilities {
    pub completion: bool,
    pub hover: bool,
    pub signature_help: bool,
    pub goto_definition: bool,
    pub goto_declaration: bool,
    pub goto_type_definition: bool,
    pub goto_implementation: bool,
    pub references: bool,
    pub rename: bool,
    pub code_action: bool,
    pub formatting: bool,
    pub range_formatting: bool,
    pub workspace_symbols: bool,
    pub diagnostics_push: bool,
}

pub fn default_client_capabilities() -> ClientCapabilities {
    ClientCapabilities {
        workspace: Some(lsp_types::WorkspaceClientCapabilities {
            apply_edit: Some(true),
            workspace_edit: None,
            did_change_configuration: None,
            did_change_watched_files: None,
            symbol: None,
            execute_command: None,
            workspace_folders: Some(true),
            configuration: Some(true),
            semantic_tokens: None,
            code_lens: None,
            file_operations: None,
            inline_value: None,
            inlay_hint: None,
            diagnostic: None,
        }),
        text_document: Some(lsp_types::TextDocumentClientCapabilities {
            synchronization: Some(lsp_types::TextDocumentSyncClientCapabilities {
                dynamic_registration: Some(false),
                will_save: Some(false),
                will_save_wait_until: Some(false),
                did_save: Some(true),
            }),
            completion: Some(lsp_types::CompletionClientCapabilities {
                dynamic_registration: Some(false),
                completion_item: Some(lsp_types::CompletionItemCapability {
                    snippet_support: Some(true),
                    commit_characters_support: Some(true),
                    documentation_format: Some(vec![
                        lsp_types::MarkupKind::Markdown,
                        lsp_types::MarkupKind::PlainText,
                    ]),
                    deprecated_support: Some(true),
                    preselect_support: Some(true),
                    tag_support: None,
                    insert_replace_support: Some(true),
                    resolve_support: None,
                    insert_text_mode_support: None,
                    label_details_support: Some(true),
                }),
                completion_item_kind: None,
                context_support: Some(true),
                insert_text_mode: Some(lsp_types::InsertTextMode::AS_IS),
                completion_list: None,
            }),
            hover: Some(lsp_types::HoverClientCapabilities {
                dynamic_registration: Some(false),
                content_format: Some(vec![
                    lsp_types::MarkupKind::Markdown,
                    lsp_types::MarkupKind::PlainText,
                ]),
            }),
            signature_help: Some(lsp_types::SignatureHelpClientCapabilities {
                dynamic_registration: Some(false),
                signature_information: Some(lsp_types::SignatureInformationSettings {
                    documentation_format: Some(vec![
                        lsp_types::MarkupKind::Markdown,
                        lsp_types::MarkupKind::PlainText,
                    ]),
                    parameter_information: Some(lsp_types::ParameterInformationSettings {
                        label_offset_support: Some(true),
                    }),
                    active_parameter_support: Some(true),
                }),
                context_support: Some(true),
            }),
            declaration: None,
            definition: None,
            type_definition: None,
            implementation: None,
            references: Some(lsp_types::DynamicRegistrationClientCapabilities {
                dynamic_registration: Some(false),
            }),
            document_highlight: None,
            document_symbol: None,
            code_action: Some(lsp_types::CodeActionClientCapabilities {
                dynamic_registration: Some(false),
                code_action_literal_support: None,
                is_preferred_support: Some(true),
                disabled_support: Some(true),
                data_support: Some(true),
                resolve_support: None,
                honors_change_annotations: Some(false),
            }),
            code_lens: None,
            document_link: None,
            color_provider: None,
            formatting: Some(lsp_types::DynamicRegistrationClientCapabilities {
                dynamic_registration: Some(false),
            }),
            range_formatting: Some(lsp_types::DynamicRegistrationClientCapabilities {
                dynamic_registration: Some(false),
            }),
            on_type_formatting: None,
            rename: Some(lsp_types::RenameClientCapabilities {
                dynamic_registration: Some(false),
                prepare_support: Some(true),
                prepare_support_default_behavior: None,
                honors_change_annotations: Some(false),
            }),
            publish_diagnostics: Some(lsp_types::PublishDiagnosticsClientCapabilities {
                related_information: Some(true),
                tag_support: None,
                version_support: Some(true),
                code_description_support: Some(true),
                data_support: Some(true),
            }),
            folding_range: None,
            selection_range: None,
            linked_editing_range: None,
            call_hierarchy: None,
            semantic_tokens: None,
            moniker: None,
            type_hierarchy: None,
            inline_value: None,
            inlay_hint: None,
            diagnostic: None,
        }),
        window: Some(lsp_types::WindowClientCapabilities {
            work_done_progress: Some(true),
            show_message: None,
            show_document: None,
        }),
        general: Some(lsp_types::GeneralClientCapabilities {
            stale_request_support: None,
            regular_expressions: None,
            markdown: None,
            position_encodings: Some(vec![lsp_types::PositionEncodingKind::UTF16]),
        }),
        experimental: None,
        notebook_document: None,
    }
}

pub fn negotiate(server: &InitializeResult) -> NegotiatedCapabilities {
    let capabilities = &server.capabilities;
    NegotiatedCapabilities {
        completion: capabilities.completion_provider.is_some(),
        hover: hover_supported(capabilities),
        signature_help: capabilities.signature_help_provider.is_some(),
        goto_definition: one_of_supported(&capabilities.definition_provider),
        goto_declaration: declaration_supported(&capabilities.declaration_provider),
        goto_type_definition: type_definition_supported(&capabilities.type_definition_provider),
        goto_implementation: implementation_supported(&capabilities.implementation_provider),
        references: one_of_supported(&capabilities.references_provider),
        rename: one_of_supported(&capabilities.rename_provider),
        code_action: code_action_supported(&capabilities.code_action_provider),
        formatting: one_of_supported(&capabilities.document_formatting_provider),
        range_formatting: one_of_supported(&capabilities.document_range_formatting_provider),
        workspace_symbols: one_of_supported(&capabilities.workspace_symbol_provider),
        diagnostics_push: true,
    }
}

fn hover_supported(capabilities: &ServerCapabilities) -> bool {
    match capabilities.hover_provider {
        Some(HoverProviderCapability::Simple(value)) => value,
        Some(HoverProviderCapability::Options(_)) => true,
        None => false,
    }
}

fn one_of_supported<T>(value: &Option<OneOf<bool, T>>) -> bool {
    match value {
        Some(OneOf::Left(enabled)) => *enabled,
        Some(OneOf::Right(_)) => true,
        None => false,
    }
}

fn declaration_supported(value: &Option<DeclarationCapability>) -> bool {
    match value {
        Some(DeclarationCapability::Simple(enabled)) => *enabled,
        Some(DeclarationCapability::RegistrationOptions(_)) => true,
        Some(DeclarationCapability::Options(_)) => true,
        None => false,
    }
}

fn type_definition_supported(value: &Option<TypeDefinitionProviderCapability>) -> bool {
    match value {
        Some(TypeDefinitionProviderCapability::Simple(enabled)) => *enabled,
        Some(TypeDefinitionProviderCapability::Options(_)) => true,
        None => false,
    }
}

fn implementation_supported(value: &Option<ImplementationProviderCapability>) -> bool {
    match value {
        Some(ImplementationProviderCapability::Simple(enabled)) => *enabled,
        Some(ImplementationProviderCapability::Options(_)) => true,
        None => false,
    }
}

fn code_action_supported(value: &Option<CodeActionProviderCapability>) -> bool {
    match value {
        Some(CodeActionProviderCapability::Simple(enabled)) => *enabled,
        Some(CodeActionProviderCapability::Options(_)) => true,
        None => false,
    }
}
