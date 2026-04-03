# Forms Nodes

Declarative forms with compiler-generated validation, progressive enhancement, and accessible markup. The compiler emits native `<form>` elements that work without JavaScript, then layers on client-side validation and enhanced submission handling.

## FormNode

Top-level form declaration containing fields, validation, and submission configuration.

| Field                  | Type                    | Required | Description                                  |
|------------------------|-------------------------|----------|----------------------------------------------|
| node_id                | string                  | yes      | Unique identifier                            |
| name                   | string                  | no       | Human-readable name                          |
| fields                 | [FormField]             | yes      | List of form fields                          |
| field_groups           | [FormFieldGroup]        | no       | Logical groupings of fields (rendered as fieldset) |
| cross_validations      | [CrossFieldValidation]  | no       | Validations spanning multiple fields         |
| validation_mode        | ValidationMode          | no       | OnSubmit, OnBlur, OnChange, OnBlurThenChange (default) |
| submission             | FormSubmission          | yes      | Submission handling configuration            |
| initial_values_node_id | string                  | no       | DataNode providing initial values (edit forms)|
| autosave               | AutosaveConfig          | no       | Draft persistence configuration              |
| semantic_node_id       | string                  | no       | Reference to a SemanticNode                  |

### ValidationMode Values

| Value            | Description                                              |
|------------------|----------------------------------------------------------|
| OnSubmit         | Validate all fields on submit only                       |
| OnBlur           | Validate each field on blur                              |
| OnChange         | Validate on every keystroke                              |
| OnBlurThenChange | Validate on blur, then on change after first error (default) |

```json
{
  "node_id": "contact-form",
  "name": "contact",
  "fields": [
    {
      "name": "email",
      "field_type": "Email",
      "label": "Email address",
      "autocomplete": "Email",
      "validations": [
        { "rule_type": "Required", "message": "Email is required" },
        { "rule_type": "Email", "message": "Enter a valid email" }
      ]
    },
    {
      "name": "message",
      "field_type": "Textarea",
      "label": "Message",
      "validations": [
        { "rule_type": "Required", "message": "Message is required" },
        { "rule_type": "MinLength", "value": "10", "message": "At least 10 characters" }
      ]
    }
  ],
  "validation_mode": "OnBlurThenChange",
  "submission": {
    "action_node_id": "submit-contact",
    "encoding": "Json",
    "progressive": true,
    "success_redirect": "/thank-you"
  }
}
```

## FormField

Individual form field with type, label, validation, and accessibility attributes.

| Field             | Type               | Required | Description                                     |
|-------------------|--------------------|----------|-------------------------------------------------|
| name              | string             | yes      | Field identifier (form submission key)          |
| field_type        | FormFieldType      | no       | Text (default), Email, Password, Number, Tel, Url, Search, Select, MultiSelect, Checkbox, Radio, Textarea, File, Date, Time, DateTimeLocal, Hidden, Color, Range |
| label             | string             | yes      | Visible label (required for accessibility)      |
| placeholder       | string             | no       | Placeholder text                                |
| description       | string             | no       | Help text below the field (aria-describedby)    |
| initial_value     | string             | no       | Default value                                   |
| validations       | [ValidationRule]   | no       | Client-side validation rules                    |
| async_validations | [AsyncValidation]  | no       | Server-side async validations                   |
| options           | [SelectOption]     | no       | Options for Select and Radio fields             |
| visible_when      | string             | no       | Conditional visibility expression               |
| disabled_when     | string             | no       | Conditional disable expression                  |
| autocomplete      | AutocompleteHint   | no       | Browser autocomplete hint (default Off)         |
| accept            | string             | no       | Accepted MIME types for File fields             |
| max_file_size     | uint32             | no       | Max file size in bytes for File fields          |
| multiple          | bool               | no       | Allow multiple files (default false)            |
| step              | float32            | no       | Step value for Number and Range fields          |
| semantic_node_id  | string             | no       | Reference to a SemanticNode                     |

### AutocompleteHint Values

`Off`, `On`, `Name`, `GivenName`, `FamilyName`, `Email`, `Username`, `NewPassword`, `CurrentPassword`, `Tel`, `StreetAddress`, `City`, `State`, `PostalCode`, `Country`, `CreditCardNumber`, `CreditCardExp`, `CreditCardCsc`

```json
{
  "name": "password",
  "field_type": "Password",
  "label": "Password",
  "autocomplete": "NewPassword",
  "validations": [
    { "rule_type": "Required", "message": "Password is required" },
    { "rule_type": "MinLength", "value": "8", "message": "At least 8 characters" },
    { "rule_type": "Pattern", "value": "[A-Z]", "message": "Must contain an uppercase letter" }
  ]
}
```

## ValidationRule

A single client-side validation constraint applied to a FormField.

| Field     | Type           | Required | Description                                     |
|-----------|----------------|----------|-------------------------------------------------|
| rule_type | ValidationType | no       | Required, MinLength, MaxLength, Pattern, Min, Max, Email, Url, Custom |
| value     | string         | no       | Parameter for the rule (e.g., "8" for MinLength)|
| message   | string         | yes      | Error message displayed on validation failure   |

```json
{ "rule_type": "MaxLength", "value": "500", "message": "Maximum 500 characters" }
```

## FormSubmission

Configures how the form is submitted and what happens on success or failure.

| Field                  | Type         | Required | Description                                  |
|------------------------|--------------|----------|----------------------------------------------|
| action_node_id         | string       | yes      | ActionNode that handles submission           |
| encoding               | FormEncoding | no       | UrlEncoded, Multipart, Json (default)        |
| progressive            | bool         | no       | Works without JS via native form action (default true) |
| success_event          | string       | no       | StateMachine event to fire on success        |
| success_state_machine  | string       | no       | Target StateMachine for success event        |
| success_redirect       | string       | no       | Route to navigate to on success              |
| error_display          | string       | no       | "field" (map to fields) or "summary" (error list) |

## FormFieldGroup

Logical grouping of fields, rendered as `<fieldset>` with `<legend>`.

| Field       | Type     | Required | Description                         |
|-------------|----------|----------|-------------------------------------|
| label       | string   | yes      | Group label (rendered as legend)    |
| field_names | [string] | yes      | Field names belonging to this group |
| description | string   | no       | Group description                   |

## CrossFieldValidation

Validation spanning multiple fields (e.g., password confirmation).

| Field        | Type     | Required | Description                                  |
|--------------|----------|----------|----------------------------------------------|
| field_names  | [string] | yes      | Fields involved in this validation           |
| expression   | string   | yes      | Expression that must evaluate to true        |
| message      | string   | yes      | Error message on failure                     |
| target_field | string   | no       | Which field displays the error               |

```json
{
  "field_names": ["password", "confirm_password"],
  "expression": "password == confirm_password",
  "message": "Passwords must match",
  "target_field": "confirm_password"
}
```
