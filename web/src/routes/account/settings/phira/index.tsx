import { useBindPhiraMutation, useOAuthStatus, useUnbindWithOAuthMutation } from "@api/account";
import { createForm, required } from "@modular-forms/solid";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Input from "@widgets/input";
import { Show } from "solid-js";

type PhiraForm = {
  email: string;
  password: string;
};

export default function () {
  const [, { Form, Field }] = createForm<PhiraForm>();
  const status = useOAuthStatus();
  const bindMutation = useBindPhiraMutation({
    onSuccess: () => {
      status.refetch();
    },
  });
  const unbindMutation = useUnbindWithOAuthMutation({
    onSuccess: () => status.refetch(),
  });
  const binding = () => status.data?.find((item) => item.provider === "phira");

  return (
    <>
      <Title page={t("account.phira.title")} route="/account/settings/phira" />
      <div class="flex flex-col p-3 lg:p-6 w-full items-center">
        <div class="flex flex-col w-full max-w-5xl space-y-3">
          <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
            <span class="shrink-0 icon-[fluent--games-20-regular] w-5 h-5" />
            <span>{t("account.phira.title")}</span>
          </h3>
          <p class="opacity-70">{t("account.phira.description")}</p>
          <Show
            when={binding()}
            fallback={
              <Form onSubmit={(values) => bindMutation.mutate(values)} class="flex flex-col space-y-2">
                <Field name="email" validate={[required(t("account.phira.form.email.required"))]}>
                  {(field, props) => (
                    <Input
                      {...props}
                      value={field.value}
                      error={field.error}
                      title={t("account.phira.form.email.label")}
                      placeholder={t("account.phira.form.email.placeholder")}
                      autocomplete="username"
                      type="email"
                      required
                    />
                  )}
                </Field>
                <Field name="password" validate={[required(t("account.phira.form.password.required"))]}>
                  {(field, props) => (
                    <Input
                      {...props}
                      value={field.value}
                      error={field.error}
                      title={t("account.phira.form.password.label")}
                      placeholder={t("account.phira.form.password.placeholder")}
                      autocomplete="current-password"
                      type="password"
                      required
                    />
                  )}
                </Field>
                <Button
                  type="submit"
                  level="primary"
                  loading={bindMutation.isPending}
                  disabled={bindMutation.isPending}
                >
                  {t("account.phira.actions.bind")}
                </Button>
              </Form>
            }
          >
            {(item) => (
              <div class="flex flex-col space-y-3">
                <p>{t("account.phira.status.bound", { name: item().data?.name ?? item().data?.id ?? "Phira" })}</p>
                <Button
                  onClick={() => unbindMutation.mutate({ id: item().id })}
                  loading={unbindMutation.isPending}
                  disabled={unbindMutation.isPending}
                >
                  {t("account.phira.actions.unbind")}
                </Button>
              </div>
            )}
          </Show>
        </div>
      </div>
    </>
  );
}
