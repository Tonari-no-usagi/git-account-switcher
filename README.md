# Git Account Switcher
### ※日本語の説明は後半にあります。

### Git Account Switcher (gas)
Git Account Switcher (gas) is a lightweight command-line utility written in Rust, specifically designed for Windows users who manage multiple Git/GitHub accounts. It functions as a specialized Git Credential Helper that resolves the common 403 Forbidden error by providing the correct credentials based on the current directory or repository context.

### Key Features
*   Automatic account switching based on directory paths.
*   OAuth2 Device Flow authentication (GitHub Browser Login).
*   Secure credential management using Windows Credential Manager.
*   High-priority credential resolution to override system-wide defaults.
*   Temporary account override for specific Git commands.
*   Multilingual support (English and Japanese).

### Important Notes
*   **Google Apps Script (GAS) Notice**: This tool is an independent utility for Git account management and is not affiliated with Google Apps Script.
*   **Credential Helper Overwrite**: The `gas setup` command optimizes your global Git configuration. It inserts a blank helper entry before registering `gas` to ensure that system-default managers do not interfere with this tool.
*   **Security**: Sensitive data such as tokens and passwords are not stored in plain text. They are stored securely within the Windows Credential Manager.

### Installation
1.  Download the latest `gas-installer.msi` from the Releases page.
2.  Run the installer to add `gas` to your system path.
3.  Open a new terminal and run `gas setup`.

### Quick Start
1.  **Initial Setup**: Run `gas setup` to integrate the tool with your Git system.
2.  **Add Account**: Run `gas add [nickname]` and select "Browser Authentication". Follow the prompts to authorize via your web browser.
3.  **Configure Directory**: Navigate to your project folder and run `gas use`. Select the account to be associated with that folder.
4.  **Execute Git**: Run `git push` or `git fetch`. The tool automatically provides the correct credentials.

### Command Reference
*   **gas setup**: Configures Git to use gas as the primary credential helper.
*   **gas add [nickname]**: Registers a new account via Browser Authentication or manual token input.
*   **gas remove [nickname]**: Deletes an account configuration and its associated token from Windows Credential Manager.
*   **gas use [nickname]**: Links the current directory to a specific account.
*   **gas list**: Lists all registered accounts and directory rules.
*   **gas with [nickname] [command]**: Temporarily executes a command using the specified account.
*   **gas lang**: Changes the display language (English/Japanese).

### Disclaimer
This software is provided "as is", without warranty of any kind, express or implied. In no event shall the author be liable for any claim, damages, or other liability, including but not limited to data loss, unauthorized access, or misconfiguration of Git settings, arising from the use of this software. Use this tool at your own risk.

### License
MIT License

---

Git Account Switcher (gas) は、Windows環境で複数のGit/GitHubアカウントを管理・切り替えるためのRust製軽量コマンドラインユーティリティです。Git Credential Helperとして動作し、現在のディレクトリやリポジトリのコンテキストに応じて適切な認証情報を自動的に提供することで、複数アカウント運用時に頻発する 403 Forbidden エラーを根本的に解決します。

### 主な機能
*   ディレクトリパスに基づくアカウントの自動判定および切り替え。
*   OAuth2デバイスフローによるブラウザ認証（GitHubログイン）。
*   Windows資格情報マネージャーを使用した認証情報の安全な保管。
*   システム標準のマネージャーより優先して動作する認証解決ロジック。
*   特定のコマンド実行時の一時的なアカウント上書き機能。
*   日本語・英語の多言語対応。

### 注意事項
*   **Google Apps Script (GAS) に関する注意**: 本ツールはGitアカウント管理のための独立したユーティリティであり、Google Apps Scriptとは一切関係ありません。
*   **Credential Helperの上書き**: `gas setup`コマンドはGitのグローバル設定を最適化します。既存のマネージャー（GCM等）による干渉を防ぐため、gasを最優先のヘルパーとして登録します。
*   **セキュリティ**: トークンやパスワードは平文で保存されません。すべての機密情報はOS標準のWindows資格情報マネージャー内に安全に保護されます。

### インストール方法
1.  リリースページから最新の `gas-installer.msi` をダウンロードします。
2.  インストーラーを実行し、システムパスに `gas` を追加します。
3.  新しいターミナルを開き、 `gas setup` を実行します。

### クイックスタート
1.  **初期設定**: `gas setup` を実行して、Gitシステムと本ツールを連携させます。
2.  **アカウント追加**: `gas add [名前]` を実行し、「ブラウザ認証」を選択します。ブラウザが開くので、指示に従って承認します。
3.  **ディレクトリ設定**: プロジェクトフォルダに移動し、 `gas use` を実行して使用するアカウントを選択します。
4.  **Gitの実行**: 通常通り `git push` や `git fetch` を行います。認証はバックグラウンドで自動的に処理されます。

### コマンド一覧
*   **gas setup**: gasを最優先の認証ヘルパーとしてGitに登録します。
*   **gas add [名前]**: ブラウザ認証または手動入力により、新しいアカウントを登録します。
*   **gas remove [名前]**: 設定からアカウントを削除し、Windows資格情報マネージャー内のトークンも消去します。
*   **gas use [名前]**: 現在のディレクトリと特定のアカウントを紐付けます。
*   **gas list**: 登録済みのアカウントと設定ルールの一覧を表示します。
*   **gas with [名前] [コマンド]**: 設定を変更せず、今回のみ指定したアカウントを使用してGitコマンドを実行します。
*   **gas lang**: 表示言語（日本語/英語）を切り替えます。

### 免責事項
本ソフトウェアは「現状のまま」提供され、明示的か黙示的かを問わず、いかなる種類の保証も行いません。本ツールの使用過程で生じたデータの損失、不正アクセス、Git設定の不整合を含むいかなる損害についても、作者は一切の責任を負いません。本ツールの利用はすべて自己責任で行ってください。

### ライセンス
MIT License