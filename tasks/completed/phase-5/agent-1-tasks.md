# Agent Tasks: Infrastructure Agent

## Agent Role

**Primary Focus:** Set up open source infrastructure, licensing, and community files to prepare the project for public release

## Key Responsibilities

- Create and configure all licensing files
- Update project metadata for crate.io publication
- Establish community guidelines and templates
- Ensure legal compliance for open source release

## Assigned Tasks

### From Original Task List

- [ ] 1.0 Prepare Open Source Infrastructure and Licensing
  - [ ] 1.1 Create and configure license file
    - [ ] 1.1.1 Download MIT license template
    - [ ] 1.1.2 Update copyright year and holder information
    - [ ] 1.1.3 Save as LICENSE in project root
  - [ ] 1.2 Update Cargo.toml with required metadata
    - [ ] 1.2.1 Change crate name from "backend" to "ai-workflow-engine"
    - [ ] 1.2.2 Update version to 0.5.0 to match git tags
    - [ ] 1.2.3 Change edition from "2024" to "2021"
    - [ ] 1.2.4 Add all required fields (authors, license, description, repository, homepage, documentation, keywords, categories)
    - [ ] 1.2.5 Add package.metadata.docs.rs configuration
  - [ ] 1.3 Create community files
    - [ ] 1.3.1 Write CONTRIBUTING.md with contribution guidelines
    - [ ] 1.3.2 Create CODE_OF_CONDUCT.md using Contributor Covenant
    - [ ] 1.3.3 Create SECURITY.md for vulnerability reporting
    - [ ] 1.3.4 Create CHANGELOG.md with initial version history
  - [ ] 1.4 Set up GitHub templates
    - [ ] 1.4.1 Create .github/ISSUE_TEMPLATE/bug_report.md
    - [ ] 1.4.2 Create .github/ISSUE_TEMPLATE/feature_request.md
    - [ ] 1.4.3 Create .github/PULL_REQUEST_TEMPLATE.md
  - [ ] 1.5 Update README.md
    - [ ] 1.5.1 Replace placeholder GitHub URLs with actual repository links
    - [ ] 1.5.2 Update badge URLs with correct repository information
    - [ ] 1.5.3 Add installation instructions for crate usage

## Relevant Files

- `Cargo.toml` - Root workspace configuration requiring metadata updates
- `LICENSE` - MIT license file to be created
- `CONTRIBUTING.md` - Contribution guidelines to be created
- `CODE_OF_CONDUCT.md` - Community code of conduct to be created  
- `SECURITY.md` - Security vulnerability reporting guidelines to be created
- `CHANGELOG.md` - Change log to be created for version tracking
- `.github/ISSUE_TEMPLATE/` - Issue templates directory to be created
- `.github/PULL_REQUEST_TEMPLATE.md` - PR template to be created
- `README.md` - Main project documentation requiring URL updates

## Dependencies

### Prerequisites (What this agent needs before starting)

- None - This agent can start immediately

### Provides to Others (What this agent delivers)

- **To Architecture Agent:** Updated Cargo.toml with new crate name and metadata (needed before workspace restructuring)
- **To Documentation & DevOps Agent:** Completed community files for reference in documentation
- **To All Agents:** License information for file headers if needed

## Handoff Points

- **After Task 1.2:** Notify Architecture Agent that Cargo.toml metadata is updated and crate rename is complete
- **After Task 1.3:** Notify Documentation & DevOps Agent that community files are ready for reference
- **After Task 1.5:** Notify all agents that README.md has updated repository URLs

## Testing Responsibilities

- Verify all URLs in README.md are valid and accessible
- Ensure Cargo.toml has valid TOML syntax
- Validate that all template files follow proper Markdown formatting
- Check that license file matches standard MIT format

## Notes

- Use actual repository owner information when updating copyright and URLs
- Ensure all metadata fields in Cargo.toml follow crates.io guidelines
- Keep CONTRIBUTING.md friendly and welcoming to new contributors
- CODE_OF_CONDUCT.md should use Contributor Covenant 2.1
- CHANGELOG.md should start with version 0.5.0 and work backwards