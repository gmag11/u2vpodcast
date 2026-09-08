## ADDED Requirements

### Requirement: Action buttons are disabled while the dialog save is pending

While the Add Channel dialog's save request is in flight (including the upstream link check performed during channel creation), both the primary action button (Create/Save) and the Cancel button SHALL be disabled. Disabled buttons SHALL ignore clicks, preventing duplicate submissions and preventing the dialog from being dismissed via those buttons mid-request.

#### Scenario: Create request in flight disables both buttons

- **WHEN** the user accepts the channel creation dialog and the save request is still running (e.g. the upstream link check is in progress)
- **THEN** both the primary action button and the Cancel button are disabled and ignore clicks until the request finishes

#### Scenario: Cancel clicked during pending request has no effect

- **WHEN** the save request is in flight and the user clicks the Cancel button
- **THEN** the dialog stays open, no cancel action fires, and the request continues to completion

#### Scenario: Buttons re-enable after the request finishes

- **WHEN** the save request completes (successfully or with an error) and the pending state clears
- **THEN** both buttons are enabled again and behave normally

### Requirement: Primary button shows a spinner while the save is pending

While the dialog's save request is in flight, the primary action button SHALL replace its label text with a visible spinner indicator so the user perceives that work is in progress. When the request finishes, the label SHALL be restored.

#### Scenario: Spinner replaces the primary button label

- **WHEN** the user accepts the dialog and the save request starts
- **THEN** the primary button's label (Create Channel / Save Changes) is replaced by an animated spinner and the button remains visibly disabled

#### Scenario: Label restored after the request finishes

- **WHEN** the save request completes and the pending state clears
- **THEN** the primary button shows its normal label again and is enabled