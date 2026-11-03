/* Vulkan ICD probe inside the snap. */
#define _GNU_SOURCE
#include <dlfcn.h>
#include <stdio.h>
#include <stdlib.h>
#include <vulkan/vulkan.h>

int main(void)
{
    printf("VK_ICD_FILENAMES=%s\n", getenv("VK_ICD_FILENAMES"));
    printf("VK_LOADER_DEBUG=%s\n", getenv("VK_LOADER_DEBUG"));
    printf("LD_LIBRARY_PATH=%s\n", getenv("LD_LIBRARY_PATH"));
    printf("SNAP=%s\n", getenv("SNAP"));

    const char *f = getenv("VK_ICD_FILENAMES");
    if (f) {
        FILE *fp = fopen(f, "r");
        printf("fopen %s: %p\n", f, (void *)fp);
        if (fp) fclose(fp);
    }

    void *icd = dlopen("libvulkan_lvp.so", RTLD_NOW | RTLD_LOCAL);
    printf("dlopen libvulkan_lvp.so: %p (%s)\n", icd, icd ? "ok" : dlerror());

    uint32_t n = 0;
    VkResult r = vkEnumerateInstanceExtensionProperties(NULL, &n, NULL);
    printf("enumerate: %d (%u extensions)\n", (int)r, n);
    if (r == VK_SUCCESS && n > 0) {
        VkExtensionProperties *exts = calloc(n, sizeof(VkExtensionProperties));
        vkEnumerateInstanceExtensionProperties(NULL, &n, exts);
        for (uint32_t i = 0; i < n; i++)
            printf("  %s\n", exts[i].extensionName);
        free(exts);
    }
    return 0;
}
