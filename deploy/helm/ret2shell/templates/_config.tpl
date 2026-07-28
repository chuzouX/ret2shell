{{- define "ret2shell.databaseHost" -}}
{{- if eq .Values.postgresql.mode "internal" -}}
{{ include "ret2shell.postgresqlName" . }}
{{- else -}}
{{ .Values.postgresql.external.host }}
{{- end -}}
{{- end -}}

{{- define "ret2shell.databasePort" -}}
{{- if eq .Values.postgresql.mode "internal" -}}
{{ .Values.postgresql.service.port }}
{{- else -}}
{{ .Values.postgresql.external.port }}
{{- end -}}
{{- end -}}

{{- define "ret2shell.databaseName" -}}
{{- if eq .Values.postgresql.mode "internal" -}}
{{ .Values.postgresql.auth.database }}
{{- else -}}
{{ .Values.postgresql.external.database }}
{{- end -}}
{{- end -}}

{{- define "ret2shell.databaseUser" -}}
{{- if eq .Values.postgresql.mode "internal" -}}
{{ .Values.postgresql.auth.username }}
{{- else -}}
{{ .Values.postgresql.external.username }}
{{- end -}}
{{- end -}}

{{- define "ret2shell.databasePassword" -}}
{{- if eq .Values.postgresql.mode "internal" -}}
{{ .Values.postgresql.auth.password }}
{{- else -}}
{{ .Values.postgresql.external.password }}
{{- end -}}
{{- end -}}

{{- define "ret2shell.databaseSslMode" -}}
{{- if eq .Values.postgresql.mode "internal" -}}
disable
{{- else -}}
{{ .Values.postgresql.external.sslMode }}
{{- end -}}
{{- end -}}

{{- define "ret2shell.cacheUrl" -}}
{{- if eq .Values.valkey.mode "internal" -}}
{{- if .Values.valkey.auth.enabled -}}
{{ printf "redis://:%s@%s:%d/0" (.Values.valkey.auth.password | urlquery) (include "ret2shell.valkeyName" .) (int .Values.valkey.service.port) }}
{{- else -}}
{{ printf "redis://%s:%d/0" (include "ret2shell.valkeyName" .) (int .Values.valkey.service.port) }}
{{- end -}}
{{- else -}}
{{ .Values.valkey.external.url }}
{{- end -}}
{{- end -}}

{{- define "ret2shell.queueHost" -}}
{{- if eq .Values.nats.mode "internal" -}}
{{ include "ret2shell.natsName" . }}
{{- else -}}
{{ .Values.nats.external.host }}
{{- end -}}
{{- end -}}

{{- define "ret2shell.queuePort" -}}
{{- if eq .Values.nats.mode "internal" -}}
{{ .Values.nats.service.port }}
{{- else -}}
{{ .Values.nats.external.port }}
{{- end -}}
{{- end -}}

{{- define "ret2shell.queueToken" -}}
{{- if eq .Values.nats.mode "internal" -}}
{{- if .Values.nats.auth.enabled -}}
{{ .Values.nats.auth.token }}
{{- end -}}
{{- else -}}
{{ .Values.nats.external.token }}
{{- end -}}
{{- end -}}

{{- define "ret2shell.queueUser" -}}
{{- if eq .Values.nats.mode "external" -}}
{{ .Values.nats.external.user }}
{{- end -}}
{{- end -}}

{{- define "ret2shell.queuePassword" -}}
{{- if eq .Values.nats.mode "external" -}}
{{ .Values.nats.external.password }}
{{- end -}}
{{- end -}}

{{- define "ret2shell.queueTls" -}}
{{- if eq .Values.nats.mode "external" -}}
{{- if .Values.nats.external.tls -}}true{{- else -}}false{{- end -}}
{{- else -}}
false
{{- end -}}
{{- end -}}

{{- define "ret2shell.victoriaUrl" -}}
{{- if eq .Values.victoriaLogs.mode "internal" -}}
{{ printf "http://%s:%d" (include "ret2shell.victoriaLogsName" .) (int .Values.victoriaLogs.service.port) }}
{{- else -}}
{{ .Values.victoriaLogs.external.url }}
{{- end -}}
{{- end -}}

{{- define "ret2shell.platformConfigToml" -}}
[auditor]
sensitive_word_list = '/etc/ret2shell/blocked.txt'

[auth]
buffer_time = {{ .Values.platform.config.auth.bufferTime }}
expires_time = {{ .Values.platform.config.auth.expiresTime }}
signing_key = {{ .Values.platform.config.auth.signingKey | quote }}

[cache]
url = {{ include "ret2shell.cacheUrl" . | quote }}

[captcha]
enabled = {{ .Values.platform.config.captcha.enabled }}
difficulty = {{ .Values.platform.config.captcha.difficulty }}
validator = {{ .Values.platform.config.captcha.validator | quote }}

[database]
host = {{ include "ret2shell.databaseHost" . | quote }}
port = {{ include "ret2shell.databasePort" . }}
db = {{ include "ret2shell.databaseName" . | quote }}
user = {{ include "ret2shell.databaseUser" . | quote }}
password = {{ include "ret2shell.databasePassword" . | quote }}
ssl_mode = {{ include "ret2shell.databaseSslMode" . | quote }}

[email]
enabled = {{ .Values.platform.config.email.enabled }}
host = {{ .Values.platform.config.email.host | quote }}
port = {{ .Values.platform.config.email.port }}
sender = {{ .Values.platform.config.email.sender | quote }}
username = {{ .Values.platform.config.email.username | quote }}
password = {{ .Values.platform.config.email.password | quote }}
tls = {{ .Values.platform.config.email.tls | quote }}
{{- if .Values.platform.config.email.senderAddress }}
sender_address = {{ .Values.platform.config.email.senderAddress | quote }}
{{- end }}

[logging]
directory = '/var/lib/ret2shell/log'
level = {{ .Values.platform.config.logging.level | quote }}
files_kept = {{ .Values.platform.config.logging.filesKept }}
compress = {{ .Values.platform.config.logging.compress }}
{{- if ne .Values.victoriaLogs.mode "disabled" }}
victoria = {{ include "ret2shell.victoriaUrl" . | quote }}
{{- end }}

[media]
anti_theft = true
limit = 100
path = '/var/lib/ret2shell/media'

[queue]
host = {{ include "ret2shell.queueHost" . | quote }}
port = {{ include "ret2shell.queuePort" . }}
{{- if include "ret2shell.queueToken" . }}
token = {{ include "ret2shell.queueToken" . | quote }}
{{- end }}
{{- if include "ret2shell.queueUser" . }}
user = {{ include "ret2shell.queueUser" . | quote }}
{{- end }}
{{- if include "ret2shell.queuePassword" . }}
password = {{ include "ret2shell.queuePassword" . | quote }}
{{- end }}
tls = {{ include "ret2shell.queueTls" . }}

[server]
host = '0.0.0.0'
port = {{ .Values.platform.service.port }}
api_base_path = {{ .Values.platform.config.server.apiBasePath | quote }}
cors_origins = {{ .Values.platform.config.server.corsOrigins | quote }}
external_domain = {{ .Values.platform.config.server.externalDomain | quote }}
external_https = {{ .Values.platform.config.server.externalHttps }}
{{- if .Values.platform.config.server.name }}
name = {{ .Values.platform.config.server.name | quote }}
{{- end }}
{{- if .Values.platform.config.server.footerInfo }}
footer_info = {{ .Values.platform.config.server.footerInfo | quote }}
{{- end }}
{{- if .Values.platform.config.server.footerUrl }}
footer_url = {{ .Values.platform.config.server.footerUrl | quote }}
{{- end }}
{{- if .Values.platform.config.server.subjectInfo }}
subject_info = {{ .Values.platform.config.server.subjectInfo | quote }}
{{- end }}
{{- if .Values.platform.config.server.subjectUrl }}
subject_url = {{ .Values.platform.config.server.subjectUrl | quote }}
{{- end }}
{{- if .Values.platform.config.server.record }}
record = {{ .Values.platform.config.server.record | quote }}
{{- end }}
hide_maker = {{ .Values.platform.config.server.hideMaker }}
{{- if .Values.platform.config.server.highlightBanner }}
highlight_banner = {{ .Values.platform.config.server.highlightBanner | quote }}
{{- end }}
[server.rate_limit]
burst_limit = {{ .Values.platform.config.server.rateLimit.burstLimit }}
burst_restore_rate = {{ .Values.platform.config.server.rateLimit.burstRestoreRate }}

[server.frontend]
serve_type = 'static'
path = '/var/www/html'
{{- if .Values.platform.config.extraToml }}

{{ .Values.platform.config.extraToml }}
{{- end }}
{{- end -}}
