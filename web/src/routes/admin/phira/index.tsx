import { usePlatformConfig, useUpdatePlatformConfigMutation } from "@api/platform";
import type { Config } from "@models/config";
import { createForm, setValues } from "@modular-forms/solid";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Input from "@widgets/input";
import { createEffect, untrack } from "solid-js";

type PhiraConfigForm = {
  base_url?: string;
};

export default function () {
  const config = usePlatformConfig();
  const [form, { Form, Field }] = createForm<PhiraConfigForm>({
    initialValues: {
      base_url: "",
    },
  });
  const mutation = useUpdatePlatformConfigMutation({
    onSuccess: () => config.refetch(),
  });

  createEffect(() => {
    if (config.data)
      untrack(() => {
        setValues(form, {
          base_url: config.data.phira?.base_url || "",
        });
      });
  });

  function onSubmit(result: PhiraConfigForm) {
    if (!config.data) return;
    mutation.mutate({
      ...config.data,
      phira: {
        ...config.data.phira,
        base_url: result.base_url?.trim() || "",
      },
    } as Config);
  }

  return (
    <>
      <Title page={t("platform.phira.title")} route="/admin/phira" />
      <div class="flex-1 flex flex-col items-center p-3 lg:p-6">
        <Form onSubmit={onSubmit} class="w-full max-w-5xl flex flex-col space-y-2">
          <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
            <span class="shrink-0 icon-[fluent--cloud-20-regular] w-5 h-5" />
            <span>{t("platform.phira.title")}</span>
          </h3>
          <Field name="base_url">
            {(field, props) => (
              <Input
                icon={<span class="shrink-0 icon-[fluent--link-20-regular] w-5 h-5" />}
                placeholder={t("platform.form.phiraBaseUrl.placeholder")}
                title={t("platform.form.phiraBaseUrl.label")}
                {...props}
                value={field.value}
                error={field.error}
                type="url"
              />
            )}
          </Field>
          <Button
            type="submit"
            level="primary"
            class="mt-4!"
            loading={config.isLoading || mutation.isPending}
            disabled={config.isLoading || mutation.isPending}
          >
            {t("general.actions.save.title")}
          </Button>
        </Form>
      </div>
    </>
  );
}
